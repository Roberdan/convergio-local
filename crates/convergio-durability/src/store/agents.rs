//! Durable registry for agent identities.

use crate::error::{DurabilityError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Agent registration input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAgent {
    /// Stable unique agent id for this worker.
    pub id: String,
    /// Host/tool kind, for example `claude`, `copilot`, or `shell`.
    pub kind: String,
    /// Optional display name.
    #[serde(default)]
    pub name: Option<String>,
    /// Optional host/session label.
    #[serde(default)]
    pub host: Option<String>,
    /// Declared capabilities or skills.
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Free-form metadata.
    #[serde(default = "default_metadata")]
    pub metadata: Value,
}

/// Agent heartbeat input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHeartbeat {
    /// Optional current task id.
    #[serde(default)]
    pub current_task_id: Option<String>,
    /// Optional status; defaults to `working` when task is present, otherwise `idle`.
    #[serde(default)]
    pub status: Option<String>,
}

/// Persisted agent registry row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecord {
    /// Stable unique agent id.
    pub id: String,
    /// Host/tool kind.
    pub kind: String,
    /// Optional display name.
    pub name: Option<String>,
    /// Optional host/session label.
    pub host: Option<String>,
    /// Agent status: `idle`, `working`, `unhealthy`, or `terminated`.
    pub status: String,
    /// Declared capabilities or skills.
    pub capabilities: Vec<String>,
    /// Optional current task id.
    pub current_task_id: Option<String>,
    /// Free-form metadata.
    pub metadata: Value,
    /// Last heartbeat timestamp.
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
}

/// Access to durable agent identities.
#[derive(Clone)]
pub struct AgentStore {
    pool: convergio_db::Pool,
}

impl AgentStore {
    /// Wrap a pool.
    pub fn new(pool: convergio_db::Pool) -> Self {
        Self { pool }
    }

    /// Register or refresh an agent identity.
    pub async fn register(&self, input: NewAgent) -> Result<AgentRecord> {
        validate_agent_id(&input.id)?;
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO agents \
             (id, kind, name, host, status, capabilities, current_task_id, metadata, created_at, updated_at) \
             VALUES (?, ?, ?, ?, 'idle', ?, NULL, ?, ?, ?) \
             ON CONFLICT(id) DO UPDATE SET kind = excluded.kind, name = excluded.name, \
             host = excluded.host, capabilities = excluded.capabilities, \
             metadata = excluded.metadata, status = 'idle', updated_at = excluded.updated_at",
        )
        .bind(&input.id)
        .bind(&input.kind)
        .bind(&input.name)
        .bind(&input.host)
        .bind(serde_json::to_string(&input.capabilities)?)
        .bind(serde_json::to_string(&input.metadata)?)
        .bind(&now)
        .bind(&now)
        .execute(self.pool.inner())
        .await?;
        self.get(&input.id).await
    }

    /// Record an agent heartbeat.
    pub async fn heartbeat(&self, agent_id: &str, input: AgentHeartbeat) -> Result<AgentRecord> {
        let status = input.status.unwrap_or_else(|| {
            if input.current_task_id.is_some() {
                "working".into()
            } else {
                "idle".into()
            }
        });
        validate_status(&status)?;
        let now = Utc::now().to_rfc3339();
        let rows = sqlx::query(
            "UPDATE agents SET status = ?, current_task_id = ?, last_heartbeat_at = ?, \
             updated_at = ? WHERE id = ?",
        )
        .bind(status)
        .bind(input.current_task_id)
        .bind(&now)
        .bind(&now)
        .bind(agent_id)
        .execute(self.pool.inner())
        .await?
        .rows_affected();
        if rows == 0 {
            return Err(DurabilityError::NotFound {
                entity: "agent",
                id: agent_id.to_string(),
            });
        }
        self.get(agent_id).await
    }

    /// Mark an agent terminated.
    pub async fn retire(&self, agent_id: &str) -> Result<AgentRecord> {
        let rows = sqlx::query(
            "UPDATE agents SET status = 'terminated', current_task_id = NULL, \
             updated_at = ? WHERE id = ?",
        )
        .bind(Utc::now().to_rfc3339())
        .bind(agent_id)
        .execute(self.pool.inner())
        .await?
        .rows_affected();
        if rows == 0 {
            return Err(DurabilityError::NotFound {
                entity: "agent",
                id: agent_id.to_string(),
            });
        }
        self.get(agent_id).await
    }

    /// Fetch one agent.
    pub async fn get(&self, agent_id: &str) -> Result<AgentRecord> {
        let row = sqlx::query_as::<_, AgentRow>(&format!("{AGENT_SELECT} WHERE id = ? LIMIT 1"))
            .bind(agent_id)
            .fetch_optional(self.pool.inner())
            .await?;
        row.map(TryInto::try_into)
            .transpose()?
            .ok_or_else(|| DurabilityError::NotFound {
                entity: "agent",
                id: agent_id.to_string(),
            })
    }

    /// List all non-terminated agents first, then historical rows.
    pub async fn list(&self) -> Result<Vec<AgentRecord>> {
        let rows = sqlx::query_as::<_, AgentRow>(&format!(
            "{AGENT_SELECT} ORDER BY status = 'terminated', updated_at DESC"
        ))
        .fetch_all(self.pool.inner())
        .await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }
}

fn validate_agent_id(id: &str) -> Result<()> {
    if id.trim().is_empty() || id.contains(char::is_whitespace) {
        return Err(DurabilityError::InvalidAgent {
            reason: "agent id must be non-empty and contain no whitespace".into(),
        });
    }
    Ok(())
}

fn validate_status(status: &str) -> Result<()> {
    if !matches!(status, "idle" | "working" | "unhealthy" | "terminated") {
        return Err(DurabilityError::InvalidAgent {
            reason: "unknown agent status".into(),
        });
    }
    Ok(())
}

fn default_metadata() -> Value {
    serde_json::json!({})
}

const AGENT_SELECT: &str = "SELECT id, kind, name, host, status, capabilities, \
     current_task_id, metadata, last_heartbeat_at, created_at, updated_at FROM agents";

#[derive(sqlx::FromRow)]
struct AgentRow {
    id: String,
    kind: String,
    name: Option<String>,
    host: Option<String>,
    status: String,
    capabilities: String,
    current_task_id: Option<String>,
    metadata: String,
    last_heartbeat_at: Option<String>,
    created_at: String,
    updated_at: String,
}

impl TryFrom<AgentRow> for AgentRecord {
    type Error = DurabilityError;
    fn try_from(row: AgentRow) -> Result<Self> {
        Ok(Self {
            id: row.id,
            kind: row.kind,
            name: row.name,
            host: row.host,
            status: row.status,
            capabilities: serde_json::from_str(&row.capabilities)?,
            current_task_id: row.current_task_id,
            metadata: serde_json::from_str(&row.metadata)?,
            last_heartbeat_at: parse_optional_time(row.last_heartbeat_at)?,
            created_at: parse_time(&row.created_at)?,
            updated_at: parse_time(&row.updated_at)?,
        })
    }
}

fn parse_optional_time(value: Option<String>) -> Result<Option<DateTime<Utc>>> {
    value.as_deref().map(parse_time).transpose()
}

fn parse_time(value: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|_| DurabilityError::NotFound {
            entity: "timestamp",
            id: value.to_string(),
        })
}
