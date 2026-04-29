//! Deterministic CRDT cell materialization.

use crate::error::{DurabilityError, Result};
use crate::store::crdt_merge_types::{clock_for_ops, common_crdt_type, merge_ops};
use crate::store::{CrdtOp, CrdtStore};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Materialized state for one CRDT entity field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtCell {
    /// Logical entity type.
    pub entity_type: String,
    /// Logical entity id.
    pub entity_id: String,
    /// Field name within the entity.
    pub field_name: String,
    /// Declared CRDT type.
    pub crdt_type: String,
    /// Materialized JSON value.
    pub value: Value,
    /// Per-actor max operation counter included in this cell.
    pub clock: Value,
    /// Conflict metadata when visible state is unresolved.
    pub conflict: Option<Value>,
    /// Last materialization timestamp.
    pub updated_at: DateTime<Utc>,
}

impl CrdtStore {
    /// Merge all operations for a single entity field into `crdt_cells`.
    pub async fn merge_cell(
        &self,
        entity_type: &str,
        entity_id: &str,
        field_name: &str,
    ) -> Result<Option<CrdtCell>> {
        let ops = self
            .ops_for_cell(entity_type, entity_id, field_name)
            .await?;
        if ops.is_empty() {
            return Ok(None);
        }
        let crdt_type = common_crdt_type(&ops)?;
        let (value, conflict) = merge_ops(&crdt_type, &ops)?;
        let clock = clock_for_ops(&ops);
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO crdt_cells \
             (entity_type, entity_id, field_name, crdt_type, value, clock, conflict, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT(entity_type, entity_id, field_name) DO UPDATE SET \
             crdt_type = excluded.crdt_type, value = excluded.value, clock = excluded.clock, \
             conflict = excluded.conflict, updated_at = excluded.updated_at",
        )
        .bind(entity_type)
        .bind(entity_id)
        .bind(field_name)
        .bind(&crdt_type)
        .bind(serde_json::to_string(&value)?)
        .bind(serde_json::to_string(&clock)?)
        .bind(conflict.as_ref().map(serde_json::to_string).transpose()?)
        .bind(now.to_rfc3339())
        .execute(self.pool().inner())
        .await?;
        self.refresh_row_clock(entity_type, entity_id).await?;

        self.get_cell(entity_type, entity_id, field_name).await
    }

    /// Fetch one materialized CRDT cell.
    pub async fn get_cell(
        &self,
        entity_type: &str,
        entity_id: &str,
        field_name: &str,
    ) -> Result<Option<CrdtCell>> {
        let row = sqlx::query_as::<_, CrdtCellRow>(
            "SELECT entity_type, entity_id, field_name, crdt_type, value, clock, conflict, \
             updated_at FROM crdt_cells \
             WHERE entity_type = ? AND entity_id = ? AND field_name = ? LIMIT 1",
        )
        .bind(entity_type)
        .bind(entity_id)
        .bind(field_name)
        .fetch_optional(self.pool().inner())
        .await?;
        row.map(TryInto::try_into).transpose()
    }

    /// List unresolved materialized CRDT conflicts.
    pub async fn list_conflicts(&self) -> Result<Vec<CrdtCell>> {
        let rows = sqlx::query_as::<_, CrdtCellRow>(
            "SELECT entity_type, entity_id, field_name, crdt_type, value, clock, conflict, \
             updated_at FROM crdt_cells WHERE conflict IS NOT NULL \
             ORDER BY entity_type, entity_id, field_name",
        )
        .fetch_all(self.pool().inner())
        .await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    /// List unresolved materialized CRDT conflicts for one entity.
    pub async fn list_conflicts_for_entity(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<CrdtCell>> {
        let rows = sqlx::query_as::<_, CrdtCellRow>(
            "SELECT entity_type, entity_id, field_name, crdt_type, value, clock, conflict, \
             updated_at FROM crdt_cells \
             WHERE entity_type = ? AND entity_id = ? AND conflict IS NOT NULL \
             ORDER BY field_name",
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_all(self.pool().inner())
        .await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn ops_for_cell(
        &self,
        entity_type: &str,
        entity_id: &str,
        field_name: &str,
    ) -> Result<Vec<CrdtOp>> {
        let rows = sqlx::query_as::<_, CrdtOpRow>(
            "SELECT actor_id, counter, entity_type, entity_id, field_name, crdt_type, \
             op_kind, value, hlc, created_at FROM crdt_ops \
             WHERE entity_type = ? AND entity_id = ? AND field_name = ? \
             ORDER BY actor_id, counter",
        )
        .bind(entity_type)
        .bind(entity_id)
        .bind(field_name)
        .fetch_all(self.pool().inner())
        .await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn refresh_row_clock(&self, entity_type: &str, entity_id: &str) -> Result<()> {
        let rows = sqlx::query_as::<_, ActorClockRow>(
            "SELECT actor_id, MAX(counter) AS counter FROM crdt_ops \
             WHERE entity_type = ? AND entity_id = ? GROUP BY actor_id ORDER BY actor_id",
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_all(self.pool().inner())
        .await?;
        let clock = rows
            .into_iter()
            .map(|row| (row.actor_id, row.counter))
            .collect::<BTreeMap<_, _>>();
        sqlx::query(
            "INSERT INTO crdt_row_clocks (entity_type, entity_id, clock, updated_at) \
             VALUES (?, ?, ?, ?) \
             ON CONFLICT(entity_type, entity_id) DO UPDATE SET \
             clock = excluded.clock, updated_at = excluded.updated_at",
        )
        .bind(entity_type)
        .bind(entity_id)
        .bind(serde_json::to_string(&clock)?)
        .bind(Utc::now().to_rfc3339())
        .execute(self.pool().inner())
        .await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct ActorClockRow {
    actor_id: String,
    counter: i64,
}

#[derive(sqlx::FromRow)]
struct CrdtCellRow {
    entity_type: String,
    entity_id: String,
    field_name: String,
    crdt_type: String,
    value: String,
    clock: String,
    conflict: Option<String>,
    updated_at: String,
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

impl TryFrom<CrdtCellRow> for CrdtCell {
    type Error = DurabilityError;

    fn try_from(row: CrdtCellRow) -> Result<Self> {
        Ok(Self {
            entity_type: row.entity_type,
            entity_id: row.entity_id,
            field_name: row.field_name,
            crdt_type: row.crdt_type,
            value: serde_json::from_str(&row.value)?,
            clock: serde_json::from_str(&row.clock)?,
            conflict: row
                .conflict
                .map(|conflict| serde_json::from_str(&conflict))
                .transpose()?,
            updated_at: parse_ts(&row.updated_at)?,
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
