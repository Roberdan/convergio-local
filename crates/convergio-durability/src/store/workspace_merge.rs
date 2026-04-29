//! Merge queue arbitration for accepted workspace patch proposals.

use crate::error::{DurabilityError, Result};
use crate::store::workspace_rows::parse_ts;
use crate::store::{PatchFile, PatchProposal, WorkspaceConflictRef, WorkspaceStore};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

/// Persisted merge queue item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeQueueItem {
    /// Queue item id.
    pub id: String,
    /// Patch proposal id.
    pub patch_proposal_id: String,
    /// Queue status: `pending`, `merged`, or `refused`.
    pub status: String,
    /// Monotonic queue sequence.
    pub sequence: i64,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
}

/// Result of processing one merge queue item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeOutcome {
    /// Processed queue item.
    pub item: MergeQueueItem,
    /// Patch proposal tied to the queue item.
    pub proposal: PatchProposal,
}

impl WorkspaceStore {
    /// Enqueue a validated patch proposal for serialized merge processing.
    pub async fn enqueue_patch_proposal(&self, proposal_id: &str) -> Result<MergeQueueItem> {
        let proposal = self.get_patch_proposal(proposal_id).await?;
        if proposal.status != "proposed" {
            return Err(DurabilityError::WorkspaceMergeRefused {
                kind: "invalid_proposal_state".into(),
                reason: "only proposed patches can be enqueued".into(),
            });
        }
        let now = Utc::now();
        let id = Uuid::new_v4().to_string();
        let next_sequence: i64 =
            sqlx::query_scalar("SELECT COALESCE(MAX(sequence), 0) + 1 FROM merge_queue")
                .fetch_one(self.pool().inner())
                .await?;
        sqlx::query(
            "INSERT INTO merge_queue (id, patch_proposal_id, status, sequence, created_at, updated_at) \
             VALUES (?, ?, 'pending', ?, ?, ?)",
        )
        .bind(&id)
        .bind(proposal_id)
        .bind(next_sequence)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(self.pool().inner())
        .await?;
        self.update_patch_status(proposal_id, "queued").await?;
        self.get_merge_item(&id).await
    }

    /// Process the next pending merge queue item.
    pub async fn process_next_merge(&self) -> Result<Option<MergeOutcome>> {
        let Some(item) = self.next_pending_merge().await? else {
            return Ok(None);
        };
        let proposal = self.get_patch_proposal(&item.patch_proposal_id).await?;
        if let Some(conflict) = self.merge_conflict(&item, &proposal).await? {
            self.refuse_merge(&item, &proposal, conflict).await?;
            return Err(DurabilityError::WorkspaceMergeRefused {
                kind: "same_file_conflict".into(),
                reason: "queued patch is stale after an earlier merge".into(),
            });
        }
        self.update_merge_status(&item, "merged").await?;
        let item = self.get_merge_item(&item.id).await?;
        let proposal = self.get_patch_proposal(&item.patch_proposal_id).await?;
        Ok(Some(MergeOutcome { item, proposal }))
    }

    /// List merge queue items in processing order.
    pub async fn merge_queue(&self) -> Result<Vec<MergeQueueItem>> {
        let rows = sqlx::query_as::<_, MergeQueueRow>(
            "SELECT id, patch_proposal_id, status, sequence, created_at, updated_at \
             FROM merge_queue ORDER BY sequence ASC",
        )
        .fetch_all(self.pool().inner())
        .await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn next_pending_merge(&self) -> Result<Option<MergeQueueItem>> {
        let row = sqlx::query_as::<_, MergeQueueRow>(
            "SELECT id, patch_proposal_id, status, sequence, created_at, updated_at \
             FROM merge_queue WHERE status = 'pending' ORDER BY sequence ASC LIMIT 1",
        )
        .fetch_optional(self.pool().inner())
        .await?;
        row.map(TryInto::try_into).transpose()
    }

    async fn merge_conflict(
        &self,
        item: &MergeQueueItem,
        proposal: &PatchProposal,
    ) -> Result<Option<MergeConflict>> {
        let previous = sqlx::query_as::<_, PatchFilesRow>(
            "SELECT p.file_hashes FROM merge_queue q \
             JOIN patch_proposals p ON p.id = q.patch_proposal_id \
             WHERE q.status = 'merged' AND q.sequence < ? ORDER BY q.sequence DESC",
        )
        .bind(item.sequence)
        .fetch_all(self.pool().inner())
        .await?;
        for row in previous {
            let files: Vec<PatchFile> = serde_json::from_str(&row.file_hashes)?;
            if let Some(conflict) = find_conflict(&proposal.files, &files) {
                return Ok(Some(conflict));
            }
        }
        Ok(None)
    }

    async fn refuse_merge(
        &self,
        item: &MergeQueueItem,
        proposal: &PatchProposal,
        conflict: MergeConflict,
    ) -> Result<()> {
        self.record_workspace_conflict(
            "same_file_conflict",
            "queued patch is stale after an earlier merge",
            Some(WorkspaceConflictRef {
                resource_id: None,
                lease_id: None,
                patch_proposal_id: Some(proposal.id.clone()),
            }),
        )
        .await?;
        self.update_merge_status(item, "refused").await?;
        sqlx::query(
            "UPDATE workspace_conflicts SET details = ? \
             WHERE patch_proposal_id = ? AND kind = 'same_file_conflict' AND status = 'open'",
        )
        .bind(serde_json::to_string(&json!({
            "reason": "queued patch is stale after an earlier merge",
            "path": conflict.path,
            "project": conflict.project,
            "expected_hash": conflict.expected_hash,
            "actual_hash": conflict.actual_hash,
        }))?)
        .bind(&proposal.id)
        .execute(self.pool().inner())
        .await?;
        Ok(())
    }

    async fn update_merge_status(&self, item: &MergeQueueItem, status: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE merge_queue SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status)
            .bind(&now)
            .bind(&item.id)
            .execute(self.pool().inner())
            .await?;
        self.update_patch_status(&item.patch_proposal_id, status)
            .await
    }

    pub(crate) async fn get_patch_proposal(&self, proposal_id: &str) -> Result<PatchProposal> {
        let row = sqlx::query_as::<_, PatchProposalRow>(
            "SELECT id, task_id, agent_id, base_revision, file_hashes, status, created_at, updated_at \
             FROM patch_proposals WHERE id = ? LIMIT 1",
        )
        .bind(proposal_id)
        .fetch_optional(self.pool().inner())
        .await?;
        row.map(TryInto::try_into)
            .transpose()?
            .ok_or_else(|| DurabilityError::NotFound {
                entity: "patch_proposal",
                id: proposal_id.to_string(),
            })
    }

    async fn get_merge_item(&self, id: &str) -> Result<MergeQueueItem> {
        let row = sqlx::query_as::<_, MergeQueueRow>(
            "SELECT id, patch_proposal_id, status, sequence, created_at, updated_at \
             FROM merge_queue WHERE id = ? LIMIT 1",
        )
        .bind(id)
        .fetch_optional(self.pool().inner())
        .await?;
        row.map(TryInto::try_into)
            .transpose()?
            .ok_or_else(|| DurabilityError::NotFound {
                entity: "merge_queue",
                id: id.to_string(),
            })
    }

    async fn update_patch_status(&self, proposal_id: &str, status: &str) -> Result<()> {
        sqlx::query("UPDATE patch_proposals SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status)
            .bind(Utc::now().to_rfc3339())
            .bind(proposal_id)
            .execute(self.pool().inner())
            .await?;
        Ok(())
    }
}

fn find_conflict(current: &[PatchFile], previous: &[PatchFile]) -> Option<MergeConflict> {
    current.iter().find_map(|file| {
        previous
            .iter()
            .find(|merged| same_file(file, merged) && merged.proposed_hash != file.base_hash)
            .map(|merged| MergeConflict {
                path: file.path.clone(),
                project: file.project.clone(),
                expected_hash: file.base_hash.clone(),
                actual_hash: merged.proposed_hash.clone(),
            })
    })
}

fn same_file(a: &PatchFile, b: &PatchFile) -> bool {
    a.path == b.path && a.project == b.project
}

struct MergeConflict {
    path: String,
    project: Option<String>,
    expected_hash: String,
    actual_hash: String,
}

#[derive(sqlx::FromRow)]
struct PatchProposalRow {
    id: String,
    task_id: String,
    agent_id: String,
    base_revision: String,
    file_hashes: String,
    status: String,
    created_at: String,
    updated_at: String,
}

impl TryFrom<PatchProposalRow> for PatchProposal {
    type Error = DurabilityError;
    fn try_from(row: PatchProposalRow) -> Result<Self> {
        Ok(Self {
            id: row.id,
            task_id: row.task_id,
            agent_id: row.agent_id,
            base_revision: row.base_revision,
            files: serde_json::from_str(&row.file_hashes)?,
            status: row.status,
            created_at: parse_ts(&row.created_at)?,
            updated_at: parse_ts(&row.updated_at)?,
        })
    }
}

#[derive(sqlx::FromRow)]
struct MergeQueueRow {
    id: String,
    patch_proposal_id: String,
    status: String,
    sequence: i64,
    created_at: String,
    updated_at: String,
}

impl TryFrom<MergeQueueRow> for MergeQueueItem {
    type Error = DurabilityError;
    fn try_from(row: MergeQueueRow) -> Result<Self> {
        Ok(Self {
            id: row.id,
            patch_proposal_id: row.patch_proposal_id,
            status: row.status,
            sequence: row.sequence,
            created_at: parse_ts(&row.created_at)?,
            updated_at: parse_ts(&row.updated_at)?,
        })
    }
}

#[derive(sqlx::FromRow)]
struct PatchFilesRow {
    file_hashes: String,
}
