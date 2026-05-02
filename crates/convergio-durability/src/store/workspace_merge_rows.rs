//! Row mapping for workspace merge queue persistence.

use crate::error::{DurabilityError, Result};
use crate::store::workspace_rows::parse_ts;
use crate::store::{MergeQueueItem, PatchProposal};

#[derive(sqlx::FromRow)]
pub(super) struct PatchProposalRow {
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
pub(super) struct MergeQueueRow {
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
pub(super) struct PatchFilesRow {
    pub(super) file_hashes: String,
}
