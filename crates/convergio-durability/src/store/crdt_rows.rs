//! Row mapping for CRDT actor and operation persistence.

use crate::error::{DurabilityError, Result};
use crate::store::{CrdtActor, CrdtOp};
use chrono::{DateTime, Utc};

#[derive(sqlx::FromRow)]
pub(super) struct CrdtActorRow {
    actor_id: String,
    kind: String,
    display_name: Option<String>,
    is_local: i64,
    created_at: String,
    last_seen_at: String,
}

#[derive(sqlx::FromRow)]
pub(super) struct CrdtOpRow {
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
