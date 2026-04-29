//! Pure CRDT merge functions for supported field types.

use crate::error::{DurabilityError, Result};
use crate::store::CrdtOp;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn common_crdt_type(ops: &[CrdtOp]) -> Result<String> {
    let first = ops[0].crdt_type.clone();
    if ops.iter().any(|op| op.crdt_type != first) {
        return Err(DurabilityError::MixedCrdtTypes {
            entity_type: ops[0].entity_type.clone(),
            entity_id: ops[0].entity_id.clone(),
            field_name: ops[0].field_name.clone(),
        });
    }
    Ok(first)
}

pub(super) fn merge_ops(crdt_type: &str, ops: &[CrdtOp]) -> Result<(Value, Option<Value>)> {
    validate_op_kinds(crdt_type, ops)?;
    match crdt_type {
        "lww_register" => Ok((lww_register(ops), None)),
        "mv_register" => Ok(mv_register(ops)),
        "or_set" => Ok((or_set(ops)?, None)),
        other => Err(DurabilityError::UnsupportedCrdtType {
            crdt_type: other.to_string(),
        }),
    }
}

pub(super) fn clock_for_ops(ops: &[CrdtOp]) -> Value {
    let mut clock = BTreeMap::<String, i64>::new();
    for op in ops {
        clock
            .entry(op.actor_id.clone())
            .and_modify(|counter| *counter = (*counter).max(op.counter))
            .or_insert(op.counter);
    }
    json!(clock)
}

fn lww_register(ops: &[CrdtOp]) -> Value {
    ops.iter()
        .filter(|op| op.op_kind == "set")
        .max_by(|a, b| lww_key(a).cmp(&lww_key(b)))
        .map(|op| op.value.clone())
        .unwrap_or(Value::Null)
}

fn validate_op_kinds(crdt_type: &str, ops: &[CrdtOp]) -> Result<()> {
    for op in ops {
        let supported = match crdt_type {
            "lww_register" | "mv_register" => op.op_kind == "set",
            "or_set" => matches!(op.op_kind.as_str(), "add" | "remove"),
            _ => true,
        };
        if !supported {
            return Err(DurabilityError::UnsupportedCrdtOperation {
                crdt_type: crdt_type.to_string(),
                op_kind: op.op_kind.clone(),
            });
        }
    }
    Ok(())
}

fn mv_register(ops: &[CrdtOp]) -> (Value, Option<Value>) {
    let mut latest_by_actor = BTreeMap::<String, &CrdtOp>::new();
    for op in ops.iter().filter(|op| op.op_kind == "set") {
        latest_by_actor
            .entry(op.actor_id.clone())
            .and_modify(|current| {
                if op.counter > current.counter {
                    *current = op;
                }
            })
            .or_insert(op);
    }
    let mut values = BTreeMap::<String, Value>::new();
    let mut candidates = Vec::new();
    for op in latest_by_actor.values() {
        let encoded = serde_json::to_string(&op.value).unwrap_or_default();
        values.entry(encoded).or_insert_with(|| op.value.clone());
        candidates.push(json!({
            "actor_id": op.actor_id,
            "counter": op.counter,
            "value": op.value,
        }));
    }
    if values.len() <= 1 {
        return (values.into_values().next().unwrap_or(Value::Null), None);
    }
    let materialized = json!({"values": values.into_values().collect::<Vec<_>>()});
    let conflict = json!({"type": "mv_register_conflict", "candidates": candidates});
    (materialized, Some(conflict))
}

fn or_set(ops: &[CrdtOp]) -> Result<Value> {
    let mut entries = BTreeMap::<String, BTreeMap<String, Value>>::new();
    let mut sorted = ops.iter().collect::<Vec<_>>();
    sorted.sort_by(|a, b| lww_key(a).cmp(&lww_key(b)));
    for op in sorted {
        let key = set_key(&op.value)?;
        match op.op_kind.as_str() {
            "add" => {
                entries
                    .entry(key)
                    .or_default()
                    .insert(dot(op), set_value(&op.value));
            }
            "remove" => remove_set_entries(&mut entries, &key, &op.value),
            _ => {}
        }
    }
    Ok(json!(entries
        .values()
        .flat_map(|dots| dots.values().cloned())
        .collect::<Vec<_>>()))
}

fn remove_set_entries(
    entries: &mut BTreeMap<String, BTreeMap<String, Value>>,
    key: &str,
    value: &Value,
) {
    let Some(dots) = entries.get_mut(key) else {
        return;
    };
    let observed = observed_dots(value);
    if observed.is_empty() {
        dots.clear();
    } else {
        for dot in observed {
            dots.remove(&dot);
        }
    }
}

fn lww_key(op: &CrdtOp) -> (&str, &str, i64) {
    (&op.hlc, &op.actor_id, op.counter)
}

fn dot(op: &CrdtOp) -> String {
    format!("{}:{}", op.actor_id, op.counter)
}

fn set_key(value: &Value) -> Result<String> {
    value
        .get("key")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| DurabilityError::InvalidCrdtValue {
            crdt_type: "or_set".to_string(),
            reason: "expected string field `key`".to_string(),
        })
}

fn set_value(value: &Value) -> Value {
    value.get("value").cloned().unwrap_or_else(|| value.clone())
}

fn observed_dots(value: &Value) -> BTreeSet<String> {
    value
        .get("dots")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}
