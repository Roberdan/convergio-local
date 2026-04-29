//! CRDT core storage tests.

use convergio_db::Pool;
use convergio_durability::{init, AppendOutcome, Durability, DurabilityError, NewCrdtOp};
use serde_json::json;
use tempfile::tempdir;

async fn fresh() -> (Durability, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let url = format!("sqlite://{}/state.db", dir.path().display());
    let pool = Pool::connect(&url).await.unwrap();
    init(&pool).await.unwrap();
    (Durability::new(pool), dir)
}

fn op(actor_id: &str, counter: i64) -> NewCrdtOp {
    NewCrdtOp {
        actor_id: actor_id.to_string(),
        counter,
        entity_type: "task".into(),
        entity_id: "task-1".into(),
        field_name: "title".into(),
        crdt_type: "mv_register".into(),
        op_kind: "set".into(),
        value: json!({"value": "first"}),
        hlc: "2026-04-29T00:00:00Z/0".into(),
    }
}

#[tokio::test]
async fn local_actor_is_created_once_and_reused() {
    let (dur, _dir) = fresh().await;
    let first = dur.crdt().local_actor().await.unwrap();
    let second = dur.crdt().local_actor().await.unwrap();

    assert_eq!(first.actor_id, second.actor_id);
    assert!(first.is_local);
    assert_eq!(first.kind, "local");

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM crdt_actors WHERE is_local = 1")
        .fetch_one(dur.pool().inner())
        .await
        .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn append_op_is_idempotent_for_same_actor_counter_payload() {
    let (dur, _dir) = fresh().await;
    let actor = dur.crdt().local_actor().await.unwrap();
    let counter = dur.crdt().next_counter(&actor.actor_id).await.unwrap();
    let op = op(&actor.actor_id, counter);

    let first = dur.crdt().append_op(op.clone()).await.unwrap();
    let second = dur.crdt().append_op(op).await.unwrap();

    assert_eq!(first, AppendOutcome::Inserted);
    assert_eq!(second, AppendOutcome::AlreadyPresent);
    assert_eq!(dur.crdt().next_counter(&actor.actor_id).await.unwrap(), 2);

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM crdt_ops")
        .fetch_one(dur.pool().inner())
        .await
        .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn append_op_rejects_same_actor_counter_with_different_payload() {
    let (dur, _dir) = fresh().await;
    let actor = dur.crdt().local_actor().await.unwrap();
    let mut changed = op(&actor.actor_id, 1);

    dur.crdt().append_op(changed.clone()).await.unwrap();
    changed.value = json!({"value": "conflict"});

    let err = dur.crdt().append_op(changed).await.unwrap_err();
    assert!(matches!(
        err,
        DurabilityError::CrdtOpConflict {
            actor_id: _,
            counter: 1
        }
    ));
}

#[tokio::test]
async fn imported_actor_is_created_when_appending_remote_op() {
    let (dur, _dir) = fresh().await;

    let outcome = dur.crdt().append_op(op("remote-actor", 7)).await.unwrap();
    assert_eq!(outcome, AppendOutcome::Inserted);

    let (kind, is_local): (String, i64) =
        sqlx::query_as("SELECT kind, is_local FROM crdt_actors WHERE actor_id = ?")
            .bind("remote-actor")
            .fetch_one(dur.pool().inner())
            .await
            .unwrap();
    assert_eq!(kind, "imported");
    assert_eq!(is_local, 0);
}
