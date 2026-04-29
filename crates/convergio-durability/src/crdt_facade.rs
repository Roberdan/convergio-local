//! Audited CRDT facade operations.

use crate::audit::EntityKind;
use crate::store::{AppendOutcome, CrdtCell, NewCrdtOp};
use crate::{Durability, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeSet;
use uuid::Uuid;

/// Result of importing and materializing a CRDT operation batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtImportResult {
    /// Audit/entity id for this import batch.
    pub batch_id: String,
    /// Number of newly inserted CRDT operations.
    pub inserted: usize,
    /// Number of idempotent duplicate operations already present.
    pub already_present: usize,
    /// Cells materialized after the import.
    pub merged_cells: Vec<CrdtCell>,
}

impl Durability {
    /// Import CRDT operations, materialize affected cells, and audit the batch.
    pub async fn import_crdt_ops(
        &self,
        ops: Vec<NewCrdtOp>,
        agent_id: Option<&str>,
    ) -> Result<CrdtImportResult> {
        let mut inserted = 0usize;
        let mut already_present = 0usize;
        let mut cells = BTreeSet::<(String, String, String)>::new();
        let mut op_ids = Vec::with_capacity(ops.len());

        for op in ops {
            cells.insert((
                op.entity_type.clone(),
                op.entity_id.clone(),
                op.field_name.clone(),
            ));
            op_ids.push(json!({
                "actor_id": &op.actor_id,
                "counter": op.counter,
                "entity_type": &op.entity_type,
                "entity_id": &op.entity_id,
                "field_name": &op.field_name,
            }));
            match self.crdt().append_op(op).await? {
                AppendOutcome::Inserted => inserted += 1,
                AppendOutcome::AlreadyPresent => already_present += 1,
            }
        }

        let mut merged_cells = Vec::new();
        for (entity_type, entity_id, field_name) in cells {
            if let Some(cell) = self
                .crdt()
                .merge_cell(&entity_type, &entity_id, &field_name)
                .await?
            {
                merged_cells.push(cell);
            }
        }

        let batch_id = Uuid::new_v4().to_string();
        let merged = merged_cells
            .iter()
            .map(|cell| {
                json!({
                    "entity_type": cell.entity_type,
                    "entity_id": cell.entity_id,
                    "field_name": cell.field_name,
                    "crdt_type": cell.crdt_type,
                    "conflict": cell.conflict.is_some(),
                })
            })
            .collect::<Vec<_>>();
        self.audit()
            .append(
                EntityKind::Crdt,
                &batch_id,
                "crdt.ops_imported",
                &json!({
                    "batch_id": &batch_id,
                    "inserted": inserted,
                    "already_present": already_present,
                    "ops": op_ids,
                    "merged_cells": merged,
                }),
                agent_id,
            )
            .await?;

        Ok(CrdtImportResult {
            batch_id,
            inserted,
            already_present,
            merged_cells,
        })
    }
}
