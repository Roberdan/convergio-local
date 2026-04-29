//! Workspace resource and lease store.

use crate::error::{DurabilityError, Result};
use crate::store::workspace_rows::{LeaseRow, ResourceRow};
use chrono::{DateTime, Utc};
use convergio_db::Pool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Workspace resource identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewWorkspaceResource {
    /// Resource kind: `repo`, `file`, `directory`, `symbol`, `artifact`, `ci_lane`.
    pub kind: String,
    /// Optional project/repository label.
    #[serde(default)]
    pub project: Option<String>,
    /// Resource path or stable lane name.
    pub path: String,
    /// Optional symbol name for symbol-scoped leases.
    #[serde(default)]
    pub symbol: Option<String>,
}

/// Persisted workspace resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceResource {
    /// Resource id.
    pub id: String,
    /// Resource kind.
    pub kind: String,
    /// Optional project/repository label.
    pub project: Option<String>,
    /// Resource path or lane name.
    pub path: String,
    /// Optional symbol name.
    pub symbol: Option<String>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
}

/// Input for claiming a workspace lease.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewWorkspaceLease {
    /// Resource to lease.
    pub resource: NewWorkspaceResource,
    /// Optional task id the lease supports.
    #[serde(default)]
    pub task_id: Option<String>,
    /// Agent requesting the lease.
    pub agent_id: String,
    /// Optional lease purpose.
    #[serde(default)]
    pub purpose: Option<String>,
    /// Expiration timestamp.
    pub expires_at: DateTime<Utc>,
}

/// Persisted workspace lease.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceLease {
    /// Lease id.
    pub id: String,
    /// Leased resource.
    pub resource: WorkspaceResource,
    /// Optional task id.
    pub task_id: Option<String>,
    /// Agent holding the lease.
    pub agent_id: String,
    /// Optional purpose.
    pub purpose: Option<String>,
    /// Lease status: `active`, `released`, or `expired`.
    pub status: String,
    /// Expiration timestamp.
    pub expires_at: DateTime<Utc>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Release timestamp.
    pub released_at: Option<DateTime<Utc>>,
}

/// Read/write access to workspace resources and leases.
#[derive(Clone)]
pub struct WorkspaceStore {
    pool: Pool,
}

impl WorkspaceStore {
    /// Wrap a pool.
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    pub(crate) fn pool(&self) -> &Pool {
        &self.pool
    }

    /// Create or fetch a canonical resource by identity.
    pub async fn ensure_resource(&self, input: NewWorkspaceResource) -> Result<WorkspaceResource> {
        let now = Utc::now();
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT OR IGNORE INTO workspace_resources \
             (id, kind, project, path, symbol, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&input.kind)
        .bind(&input.project)
        .bind(&input.path)
        .bind(&input.symbol)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(self.pool.inner())
        .await?;
        self.find_resource(&input)
            .await?
            .ok_or(DurabilityError::NotFound {
                entity: "workspace_resource",
                id: input.path,
            })
    }

    /// Claim a resource lease, refusing active overlapping leases.
    pub async fn claim_lease(&self, input: NewWorkspaceLease) -> Result<WorkspaceLease> {
        let now = Utc::now();
        if input.expires_at <= now {
            return Err(DurabilityError::InvalidWorkspaceLease {
                reason: "expires_at must be in the future".into(),
            });
        }
        self.expire_leases(now).await?;
        let resource = self.ensure_resource(input.resource).await?;
        if let Some(active) = self.active_lease_for_resource_id(&resource.id).await? {
            return Err(DurabilityError::WorkspaceLeaseConflict {
                resource_id: resource.id,
                lease_id: active.id,
                agent_id: active.agent_id,
            });
        }

        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO workspace_leases \
             (id, resource_id, task_id, agent_id, purpose, status, expires_at, created_at) \
             VALUES (?, ?, ?, ?, ?, 'active', ?, ?)",
        )
        .bind(&id)
        .bind(&resource.id)
        .bind(&input.task_id)
        .bind(&input.agent_id)
        .bind(&input.purpose)
        .bind(input.expires_at.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(self.pool.inner())
        .await?;
        self.get_lease(&id).await
    }

    /// Release a lease.
    pub async fn release_lease(&self, lease_id: &str) -> Result<WorkspaceLease> {
        let now = Utc::now();
        let rows = sqlx::query(
            "UPDATE workspace_leases SET status = 'released', released_at = ? \
             WHERE id = ? AND status = 'active'",
        )
        .bind(now.to_rfc3339())
        .bind(lease_id)
        .execute(self.pool.inner())
        .await?
        .rows_affected();
        if rows == 0 {
            return Err(DurabilityError::NotFound {
                entity: "workspace_lease",
                id: lease_id.to_string(),
            });
        }
        self.get_lease(lease_id).await
    }

    /// List active leases after expiring stale rows.
    pub async fn active_leases(&self) -> Result<Vec<WorkspaceLease>> {
        self.expire_leases(Utc::now()).await?;
        let rows = sqlx::query_as::<_, LeaseRow>(
            "SELECT l.id, l.task_id, l.agent_id, l.purpose, l.status, l.expires_at, \
             l.created_at, l.released_at, r.id AS resource_id, r.kind, r.project, r.path, \
             r.symbol, r.created_at AS resource_created_at, r.updated_at AS resource_updated_at \
             FROM workspace_leases l JOIN workspace_resources r ON r.id = l.resource_id \
             WHERE l.status = 'active' ORDER BY l.created_at ASC",
        )
        .fetch_all(self.pool.inner())
        .await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    /// Mark all leases expired through `now`.
    pub async fn expire_leases(&self, now: DateTime<Utc>) -> Result<u64> {
        let result = sqlx::query(
            "UPDATE workspace_leases SET status = 'expired' \
             WHERE status = 'active' AND expires_at <= ?",
        )
        .bind(now.to_rfc3339())
        .execute(self.pool.inner())
        .await?;
        Ok(result.rows_affected())
    }

    pub(crate) async fn active_lease_for_resource_id(
        &self,
        resource_id: &str,
    ) -> Result<Option<WorkspaceLease>> {
        let row = sqlx::query_as::<_, LeaseRow>(&format!(
            "{LEASE_SELECT} WHERE r.id = ? AND l.status = 'active' LIMIT 1"
        ))
        .bind(resource_id)
        .fetch_optional(self.pool.inner())
        .await?;
        row.map(TryInto::try_into).transpose()
    }

    async fn get_lease(&self, lease_id: &str) -> Result<WorkspaceLease> {
        let row = sqlx::query_as::<_, LeaseRow>(&format!("{LEASE_SELECT} WHERE l.id = ? LIMIT 1"))
            .bind(lease_id)
            .fetch_optional(self.pool.inner())
            .await?;
        row.map(TryInto::try_into)
            .transpose()?
            .ok_or_else(|| DurabilityError::NotFound {
                entity: "workspace_lease",
                id: lease_id.to_string(),
            })
    }

    async fn find_resource(
        &self,
        input: &NewWorkspaceResource,
    ) -> Result<Option<WorkspaceResource>> {
        let row = sqlx::query_as::<_, ResourceRow>(
            "SELECT id, kind, project, path, symbol, created_at, updated_at \
             FROM workspace_resources \
             WHERE kind = ? AND IFNULL(project, '') = IFNULL(?, '') \
             AND path = ? AND IFNULL(symbol, '') = IFNULL(?, '') LIMIT 1",
        )
        .bind(&input.kind)
        .bind(&input.project)
        .bind(&input.path)
        .bind(&input.symbol)
        .fetch_optional(self.pool.inner())
        .await?;
        row.map(TryInto::try_into).transpose()
    }
}

const LEASE_SELECT: &str = "SELECT l.id, l.task_id, l.agent_id, l.purpose, l.status, \
     l.expires_at, l.created_at, l.released_at, r.id AS resource_id, r.kind, r.project, \
     r.path, r.symbol, r.created_at AS resource_created_at, r.updated_at AS resource_updated_at \
     FROM workspace_leases l JOIN workspace_resources r ON r.id = l.resource_id";
