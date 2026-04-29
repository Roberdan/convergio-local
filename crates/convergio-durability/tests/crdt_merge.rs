//! CRDT merge/materialization tests.

use convergio_db::Pool;
use convergio_durability::{init, Durability, DurabilityError, NewCrdtOp};
use serde_json::json;
use tempfile::tempdir;

async fn fresh() -> (Durability, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let url = format!("sqlite://{}/state.db", dir.path().display());
    let pool = Pool::connect(&url).await.unwrap();
    init(&pool).await.unwrap();
    (Durability::new(pool), dir)
}

fn cell_op(
    actor_id: &str,
    counter: i64,
    field_name: &str,
    crdt_type: &str,
    op_kind: &str,
    value: serde_json::Value,
    hlc: &str,
) -> NewCrdtOp {
    NewCrdtOp {
        actor_id: actor_id.to_string(),
        counter,
        entity_type: "task".into(),
        entity_id: "task-1".into(),
        field_name: field_name.into(),
        crdt_type: crdt_type.into(),
        op_kind: op_kind.into(),
        value,
        hlc: hlc.into(),
    }
}

#[tokio::test]
async fn lww_register_materializes_latest_value_and_row_clock() {
    let (dur, _dir) = fresh().await;
    let local = dur.crdt().local_actor().await.unwrap();
    dur.crdt()
        .append_op(cell_op(
            &local.actor_id,
            1,
            "status",
            "lww_register",
            "set",
            json!("pending"),
            "2026-04-29T00:00:00Z/0",
        ))
        .await
        .unwrap();
    dur.crdt()
        .append_op(cell_op(
            "remote-lww",
            1,
            "status",
            "lww_register",
            "set",
            json!("done"),
            "2026-04-29T00:01:00Z/0",
        ))
        .await
        .unwrap();

    let cell = dur
        .crdt()
        .merge_cell("task", "task-1", "status")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(cell.value, json!("done"));
    assert_eq!(cell.conflict, None);
    assert_eq!(cell.clock[&local.actor_id], json!(1));
    assert_eq!(cell.clock["remote-lww"], json!(1));

    let (row_clock,): (String,) =
        sqlx::query_as("SELECT clock FROM crdt_row_clocks WHERE entity_type = ? AND entity_id = ?")
            .bind("task")
            .bind("task-1")
            .fetch_one(dur.pool().inner())
            .await
            .unwrap();
    let row_clock: serde_json::Value = serde_json::from_str(&row_clock).unwrap();
    assert_eq!(row_clock[&local.actor_id], json!(1));
    assert_eq!(row_clock["remote-lww"], json!(1));
}

#[tokio::test]
async fn different_fields_merge_without_conflicts() {
    let (dur, _dir) = fresh().await;
    dur.crdt()
        .append_op(cell_op(
            "actor-title",
            1,
            "title",
            "mv_register",
            "set",
            json!("new title"),
            "2026-04-29T00:00:00Z/0",
        ))
        .await
        .unwrap();
    dur.crdt()
        .append_op(cell_op(
            "actor-desc",
            1,
            "description",
            "mv_register",
            "set",
            json!("new description"),
            "2026-04-29T00:00:01Z/0",
        ))
        .await
        .unwrap();

    let title = dur
        .crdt()
        .merge_cell("task", "task-1", "title")
        .await
        .unwrap()
        .unwrap();
    let description = dur
        .crdt()
        .merge_cell("task", "task-1", "description")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(title.value, json!("new title"));
    assert_eq!(title.conflict, None);
    assert_eq!(description.value, json!("new description"));
    assert_eq!(description.conflict, None);
}

#[tokio::test]
async fn mv_register_same_field_different_values_surfaces_conflict() {
    let (dur, _dir) = fresh().await;
    dur.crdt()
        .append_op(cell_op(
            "actor-a",
            1,
            "title",
            "mv_register",
            "set",
            json!("alpha"),
            "2026-04-29T00:00:00Z/0",
        ))
        .await
        .unwrap();
    dur.crdt()
        .append_op(cell_op(
            "actor-b",
            1,
            "title",
            "mv_register",
            "set",
            json!("beta"),
            "2026-04-29T00:00:00Z/0",
        ))
        .await
        .unwrap();

    let cell = dur
        .crdt()
        .merge_cell("task", "task-1", "title")
        .await
        .unwrap()
        .unwrap();
    let conflict = cell.conflict.unwrap();

    assert_eq!(conflict["type"], json!("mv_register_conflict"));
    assert_eq!(conflict["candidates"].as_array().unwrap().len(), 2);
    assert_eq!(cell.value["values"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn or_set_materializes_observed_remove() {
    let (dur, _dir) = fresh().await;
    dur.crdt()
        .append_op(cell_op(
            "actor-set",
            1,
            "tags",
            "or_set",
            "add",
            json!({"key": "urgent", "value": "urgent"}),
            "2026-04-29T00:00:00Z/0",
        ))
        .await
        .unwrap();
    dur.crdt()
        .append_op(cell_op(
            "actor-set",
            2,
            "tags",
            "or_set",
            "add",
            json!({"key": "review", "value": "review"}),
            "2026-04-29T00:00:01Z/0",
        ))
        .await
        .unwrap();
    dur.crdt()
        .append_op(cell_op(
            "actor-set",
            3,
            "tags",
            "or_set",
            "remove",
            json!({"key": "urgent", "dots": ["actor-set:1"]}),
            "2026-04-29T00:00:02Z/0",
        ))
        .await
        .unwrap();

    let cell = dur
        .crdt()
        .merge_cell("task", "task-1", "tags")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(cell.value, json!(["review"]));
    assert_eq!(cell.conflict, None);
}

#[tokio::test]
async fn unsupported_crdt_operation_is_rejected() {
    let (dur, _dir) = fresh().await;
    dur.crdt()
        .append_op(cell_op(
            "actor-bad-op",
            1,
            "status",
            "lww_register",
            "increment",
            json!(1),
            "2026-04-29T00:00:00Z/0",
        ))
        .await
        .unwrap();

    let err = dur
        .crdt()
        .merge_cell("task", "task-1", "status")
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        DurabilityError::UnsupportedCrdtOperation {
            crdt_type,
            op_kind,
        } if crdt_type == "lww_register" && op_kind == "increment"
    ));
}
