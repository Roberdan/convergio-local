//! CRDT actor and operation store.

use crate::error::{DurabilityError, Result};
use chrono::{DateTime, Utc};
use convergio_db::Pool;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Stable actor identity for CRDT operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtActor {
    /// Actor UUID.
    pub actor_id: String,
    /// Actor kind: `local` or `imported`.
    pub kind: String,
    /// Optional human label.
    pub display_name: Option<String>,
    /// Whether this is the current local actor.
    pub is_local: bool,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last time this actor was observed locally.
    pub last_seen_at: DateTime<Utc>,
}

/// Input for appending a CRDT operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCrdtOp {
    /// Actor UUID.
    pub actor_id: String,
    /// Per-actor monotonic counter.
    pub counter: i64,
    /// Logical entity type.
    pub entity_type: String,
    /// Logical entity id.
    pub entity_id: String,
    /// Field name within the entity.
    pub field_name: String,
    /// Declared CRDT type.
    pub crdt_type: String,
    /// Operation kind.
    pub op_kind: String,
    /// Operation payload.
    pub value: Value,
    /// Hybrid logical clock string for debugging/order display.
    pub hlc: String,
}

/// Stored CRDT operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtOp {
    /// Actor UUID.
    pub actor_id: String,
    /// Per-actor monotonic counter.
    pub counter: i64,
    /// Logical entity type.
    pub entity_type: String,
    /// Logical entity id.
    pub entity_id: String,
    /// Field name within the entity.
    pub field_name: String,
    /// Declared CRDT type.
    pub crdt_type: String,
    /// Operation kind.
    pub op_kind: String,
    /// Operation payload.
    pub value: Value,
    /// Hybrid logical clock string.
    pub hlc: String,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Result of appending a CRDT operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppendOutcome {
    /// Operation was newly inserted.
    Inserted,
    /// Same operation was already present and matched exactly.
    AlreadyPresent,
}

/// Read/write access to CRDT actor/op tables.
#[derive(Clone)]
pub struct CrdtStore {
    pool: Pool,
}

impl CrdtStore {
    /// Wrap a pool.
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Load the local actor, creating it if needed.
    pub async fn local_actor(&self) -> Result<CrdtActor> {
        if let Some(actor) = self.find_local_actor().await? {
            return Ok(actor);
        }

        let now = Utc::now();
        let actor_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO crdt_actors \
             (actor_id, kind, display_name, is_local, created_at, last_seen_at) \
             VALUES (?, 'local', ?, 1, ?, ?)",
        )
        .bind(&actor_id)
        .bind("local")
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(self.pool.inner())
        .await?;
        self.find_local_actor()
            .await?
            .ok_or(DurabilityError::NotFound {
                entity: "crdt_actor",
                id: actor_id,
            })
    }

    /// Return the next operation counter for an actor.
    pub async fn next_counter(&self, actor_id: &str) -> Result<i64> {
        let (counter,): (i64,) =
            sqlx::query_as("SELECT COALESCE(MAX(counter), 0) + 1 FROM crdt_ops WHERE actor_id = ?")
                .bind(actor_id)
                .fetch_one(self.pool.inner())
                .await?;
        Ok(counter)
    }

    /// Append a CRDT operation. Re-appending an identical op is a no-op.
    pub async fn append_op(&self, input: NewCrdtOp) -> Result<AppendOutcome> {
        self.ensure_actor(&input.actor_id).await?;
        let now = Utc::now();
        let value = serde_json::to_string(&input.value)?;
        let result = sqlx::query(
            "INSERT OR IGNORE INTO crdt_ops \
             (actor_id, counter, entity_type, entity_id, field_name, crdt_type, \
              op_kind, value, hlc, created_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&input.actor_id)
        .bind(input.counter)
        .bind(&input.entity_type)
        .bind(&input.entity_id)
        .bind(&input.field_name)
        .bind(&input.crdt_type)
        .bind(&input.op_kind)
        .bind(&value)
        .bind(&input.hlc)
        .bind(now.to_rfc3339())
        .execute(self.pool.inner())
        .await?;

        if result.rows_affected() == 1 {
            return Ok(AppendOutcome::Inserted);
        }

        let existing = self.get_op(&input.actor_id, input.counter).await?;
        if existing.same_identity_payload(&input) {
            Ok(AppendOutcome::AlreadyPresent)
        } else {
            Err(DurabilityError::CrdtOpConflict {
                actor_id: input.actor_id,
                counter: input.counter,
            })
        }
    }

    /// Fetch a stored CRDT operation.
    pub async fn get_op(&self, actor_id: &str, counter: i64) -> Result<CrdtOp> {
        let row = sqlx::query_as::<_, CrdtOpRow>(SELECT_OP)
            .bind(actor_id)
            .bind(counter)
            .fetch_optional(self.pool.inner())
            .await?;
        row.map(TryInto::try_into)
            .transpose()?
            .ok_or_else(|| DurabilityError::NotFound {
                entity: "crdt_op",
                id: format!("{actor_id}:{counter}"),
            })
    }

    async fn find_local_actor(&self) -> Result<Option<CrdtActor>> {
        let row = sqlx::query_as::<_, CrdtActorRow>(
            "SELECT actor_id, kind, display_name, is_local, created_at, last_seen_at \
             FROM crdt_actors WHERE is_local = 1 LIMIT 1",
        )
        .fetch_optional(self.pool.inner())
        .await?;
        row.map(TryInto::try_into).transpose()
    }

    async fn ensure_actor(&self, actor_id: &str) -> Result<()> {
        let now = Utc::now();
        sqlx::query(
            "INSERT OR IGNORE INTO crdt_actors \
             (actor_id, kind, display_name, is_local, created_at, last_seen_at) \
             VALUES (?, 'imported', NULL, 0, ?, ?)",
        )
        .bind(actor_id)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(self.pool.inner())
        .await?;
        Ok(())
    }
}

const SELECT_OP: &str = "SELECT actor_id, counter, entity_type, entity_id, field_name, crdt_type, \
     op_kind, value, hlc, created_at FROM crdt_ops \
     WHERE actor_id = ? AND counter = ? LIMIT 1";

#[derive(sqlx::FromRow)]
struct CrdtActorRow {
    actor_id: String,
    kind: String,
    display_name: Option<String>,
    is_local: i64,
    created_at: String,
    last_seen_at: String,
}

#[derive(sqlx::FromRow)]
struct CrdtOpRow {
    actor_id: String,
    counter: i64,
    entity_type: String,
    entity_id: String,
    field_name: String,
    crdt_type: String,
    op_kind: String,
    value: String,
    hlc: String,
    created_at: String,
}

impl CrdtOp {
    fn same_identity_payload(&self, other: &NewCrdtOp) -> bool {
        self.entity_type == other.entity_type
            && self.entity_id == other.entity_id
            && self.field_name == other.field_name
            && self.crdt_type == other.crdt_type
            && self.op_kind == other.op_kind
            && self.value == other.value
            && self.hlc == other.hlc
    }
}

impl TryFrom<CrdtActorRow> for CrdtActor {
    type Error = DurabilityError;
    fn try_from(row: CrdtActorRow) -> Result<Self> {
        Ok(Self {
            actor_id: row.actor_id,
            kind: row.kind,
            display_name: row.display_name,
            is_local: row.is_local == 1,
            created_at: parse_ts(&row.created_at)?,
            last_seen_at: parse_ts(&row.last_seen_at)?,
        })
    }
}

impl TryFrom<CrdtOpRow> for CrdtOp {
    type Error = DurabilityError;
    fn try_from(row: CrdtOpRow) -> Result<Self> {
        Ok(Self {
            actor_id: row.actor_id,
            counter: row.counter,
            entity_type: row.entity_type,
            entity_id: row.entity_id,
            field_name: row.field_name,
            crdt_type: row.crdt_type,
            op_kind: row.op_kind,
            value: serde_json::from_str(&row.value)?,
            hlc: row.hlc,
            created_at: parse_ts(&row.created_at)?,
        })
    }
}

fn parse_ts(s: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|_| DurabilityError::NotFound {
            entity: "timestamp",
            id: s.to_string(),
        })
}
