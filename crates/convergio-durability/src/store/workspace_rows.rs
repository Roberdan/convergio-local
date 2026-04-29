//! SQL row conversion for workspace coordination tables.

use crate::error::{DurabilityError, Result};
use crate::store::{WorkspaceLease, WorkspaceResource};
use chrono::{DateTime, Utc};

#[derive(sqlx::FromRow)]
pub(super) struct ResourceRow {
    pub(super) id: String,
    pub(super) kind: String,
    pub(super) project: Option<String>,
    pub(super) path: String,
    pub(super) symbol: Option<String>,
    pub(super) created_at: String,
    pub(super) updated_at: String,
}

#[derive(sqlx::FromRow)]
pub(super) struct LeaseRow {
    pub(super) id: String,
    pub(super) task_id: Option<String>,
    pub(super) agent_id: String,
    pub(super) purpose: Option<String>,
    pub(super) status: String,
    pub(super) expires_at: String,
    pub(super) created_at: String,
    pub(super) released_at: Option<String>,
    pub(super) resource_id: String,
    pub(super) kind: String,
    pub(super) project: Option<String>,
    pub(super) path: String,
    pub(super) symbol: Option<String>,
    pub(super) resource_created_at: String,
    pub(super) resource_updated_at: String,
}

impl TryFrom<ResourceRow> for WorkspaceResource {
    type Error = DurabilityError;
    fn try_from(r: ResourceRow) -> Result<Self> {
        Ok(Self {
            id: r.id,
            kind: r.kind,
            project: r.project,
            path: r.path,
            symbol: r.symbol,
            created_at: parse_ts(&r.created_at)?,
            updated_at: parse_ts(&r.updated_at)?,
        })
    }
}

impl TryFrom<LeaseRow> for WorkspaceLease {
    type Error = DurabilityError;
    fn try_from(r: LeaseRow) -> Result<Self> {
        Ok(Self {
            id: r.id,
            resource: WorkspaceResource {
                id: r.resource_id,
                kind: r.kind,
                project: r.project,
                path: r.path,
                symbol: r.symbol,
                created_at: parse_ts(&r.resource_created_at)?,
                updated_at: parse_ts(&r.resource_updated_at)?,
            },
            task_id: r.task_id,
            agent_id: r.agent_id,
            purpose: r.purpose,
            status: r.status,
            expires_at: parse_ts(&r.expires_at)?,
            created_at: parse_ts(&r.created_at)?,
            released_at: r.released_at.as_deref().map(parse_ts).transpose()?,
        })
    }
}

pub(super) fn parse_ts(s: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|_| DurabilityError::NotFound {
            entity: "timestamp",
            id: s.to_string(),
        })
}
