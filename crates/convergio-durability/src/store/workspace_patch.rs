//! Patch proposal validation on top of workspace leases.

use crate::error::{DurabilityError, Result};
use crate::store::{NewWorkspaceResource, WorkspaceStore};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Component, Path};
use uuid::Uuid;

/// One file touched by a patch proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchFile {
    /// Relative workspace path.
    pub path: String,
    /// Optional project/repository label.
    #[serde(default)]
    pub project: Option<String>,
    /// File hash observed by the agent at task start.
    pub base_hash: String,
    /// Current canonical file hash observed before submission.
    pub current_hash: String,
    /// Hash after applying the proposal.
    pub proposed_hash: String,
}

/// Input for submitting a patch proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPatchProposal {
    /// Task this proposal belongs to.
    pub task_id: String,
    /// Submitting agent id.
    pub agent_id: String,
    /// Base VCS revision.
    pub base_revision: String,
    /// Patch/diff text.
    pub patch: String,
    /// Files touched by this patch.
    pub files: Vec<PatchFile>,
}

/// Persisted patch proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchProposal {
    /// Proposal id.
    pub id: String,
    /// Task id.
    pub task_id: String,
    /// Agent id.
    pub agent_id: String,
    /// Base VCS revision.
    pub base_revision: String,
    /// Files touched by this patch.
    pub files: Vec<PatchFile>,
    /// Proposal status.
    pub status: String,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
}

/// Persisted workspace conflict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConflict {
    /// Conflict id.
    pub id: String,
    /// Conflict kind.
    pub kind: String,
    /// Conflict status.
    pub status: String,
    /// Details payload.
    pub details: Value,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Optional foreign keys for recording a workspace conflict.
#[derive(Debug, Clone)]
pub struct WorkspaceConflictRef {
    /// Resource id.
    pub resource_id: Option<String>,
    /// Lease id.
    pub lease_id: Option<String>,
    /// Patch proposal id.
    pub patch_proposal_id: Option<String>,
}

impl WorkspaceStore {
    /// Submit a patch proposal after validating lease and hash coverage.
    pub async fn submit_patch_proposal(&self, input: NewPatchProposal) -> Result<PatchProposal> {
        if input.files.is_empty() {
            return self
                .refuse_patch("empty_patch", "patch must touch at least one file", None)
                .await;
        }
        for file in &input.files {
            self.validate_patch_file(&input, file).await?;
        }

        let now = Utc::now();
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO patch_proposals \
             (id, task_id, agent_id, base_revision, patch, file_hashes, status, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, 'proposed', ?, ?)",
        )
        .bind(&id)
        .bind(&input.task_id)
        .bind(&input.agent_id)
        .bind(&input.base_revision)
        .bind(&input.patch)
        .bind(serde_json::to_string(&input.files)?)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(self.pool().inner())
        .await?;

        Ok(PatchProposal {
            id,
            task_id: input.task_id,
            agent_id: input.agent_id,
            base_revision: input.base_revision,
            files: input.files,
            status: "proposed".into(),
            created_at: now,
            updated_at: now,
        })
    }

    /// List open workspace conflicts.
    pub async fn open_workspace_conflicts(&self) -> Result<Vec<WorkspaceConflict>> {
        let rows = sqlx::query_as::<_, ConflictRow>(
            "SELECT id, kind, status, details, created_at FROM workspace_conflicts \
             WHERE status = 'open' ORDER BY created_at ASC",
        )
        .fetch_all(self.pool().inner())
        .await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn validate_patch_file(&self, input: &NewPatchProposal, file: &PatchFile) -> Result<()> {
        if unsafe_path(&file.path) {
            return self
                .refuse_patch("path_escape", "patch path escapes workspace", None)
                .await;
        }
        if file.base_hash != file.current_hash {
            return self
                .refuse_patch("stale_base", "base_hash differs from current_hash", None)
                .await;
        }

        let resource = self
            .ensure_resource(NewWorkspaceResource {
                kind: "file".into(),
                project: file.project.clone(),
                path: file.path.clone(),
                symbol: None,
            })
            .await?;
        let active = self.active_lease_for_resource_id(&resource.id).await?;
        let Some(lease) = active else {
            return self
                .refuse_patch(
                    "missing_lease",
                    "no active lease covers a touched file",
                    Some(WorkspaceConflictRef {
                        resource_id: Some(resource.id),
                        lease_id: None,
                        patch_proposal_id: None,
                    }),
                )
                .await;
        };
        if lease.agent_id != input.agent_id {
            return self
                .refuse_patch(
                    "lease_conflict",
                    "touched file is leased by another agent",
                    Some(WorkspaceConflictRef {
                        resource_id: Some(resource.id),
                        lease_id: Some(lease.id),
                        patch_proposal_id: None,
                    }),
                )
                .await;
        }
        Ok(())
    }

    async fn refuse_patch<T>(
        &self,
        kind: &str,
        reason: &str,
        refs: Option<WorkspaceConflictRef>,
    ) -> Result<T> {
        self.record_workspace_conflict(kind, reason, refs).await?;
        Err(DurabilityError::WorkspacePatchRefused {
            kind: kind.into(),
            reason: reason.into(),
        })
    }

    async fn record_workspace_conflict(
        &self,
        kind: &str,
        reason: &str,
        refs: Option<WorkspaceConflictRef>,
    ) -> Result<()> {
        let refs = refs.unwrap_or(WorkspaceConflictRef {
            resource_id: None,
            lease_id: None,
            patch_proposal_id: None,
        });
        sqlx::query(
            "INSERT INTO workspace_conflicts \
             (id, resource_id, lease_id, patch_proposal_id, kind, status, details, created_at) \
             VALUES (?, ?, ?, ?, ?, 'open', ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(refs.resource_id)
        .bind(refs.lease_id)
        .bind(refs.patch_proposal_id)
        .bind(kind)
        .bind(serde_json::to_string(&json!({"reason": reason}))?)
        .bind(Utc::now().to_rfc3339())
        .execute(self.pool().inner())
        .await?;
        Ok(())
    }
}

fn unsafe_path(path: &str) -> bool {
    let path = Path::new(path);
    path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
}

#[derive(sqlx::FromRow)]
struct ConflictRow {
    id: String,
    kind: String,
    status: String,
    details: String,
    created_at: String,
}

impl TryFrom<ConflictRow> for WorkspaceConflict {
    type Error = DurabilityError;
    fn try_from(row: ConflictRow) -> Result<Self> {
        Ok(Self {
            id: row.id,
            kind: row.kind,
            status: row.status,
            details: serde_json::from_str(&row.details)?,
            created_at: DateTime::parse_from_rfc3339(&row.created_at)
                .map(|d| d.with_timezone(&Utc))
                .map_err(|_| DurabilityError::NotFound {
                    entity: "timestamp",
                    id: row.created_at,
                })?,
        })
    }
}
