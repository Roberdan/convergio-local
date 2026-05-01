//! ADR-0025 — system.* topic family with `plan_id IS NULL`.

use convergio_bus::{init, Bus, BusError, NewMessage, NewSystemMessage};
use convergio_db::Pool;
use serde_json::json;
use tempfile::tempdir;

async fn fresh_bus() -> (Bus, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let url = format!("sqlite://{}/state.db", dir.path().display());
    let pool = Pool::connect(&url).await.unwrap();
    init(&pool).await.unwrap();
    (Bus::new(pool), dir)
}

#[tokio::test]
async fn publish_system_then_poll_system() {
    let (bus, _dir) = fresh_bus().await;

    let m = bus
        .publish_system(NewSystemMessage {
            topic: "system.session-events".into(),
            sender: Some("agent-A".into()),
            payload: json!({"kind": "agent.attached", "tty": "ttys004"}),
        })
        .await
        .unwrap();
    assert!(
        m.plan_id.is_none(),
        "system messages must persist with plan_id NULL"
    );
    assert_eq!(m.topic, "system.session-events");

    let polled = bus
        .poll_system("system.session-events", 0, 10)
        .await
        .unwrap();
    assert_eq!(polled.len(), 1);
    assert_eq!(polled[0].id, m.id);
    assert!(polled[0].plan_id.is_none());
}

#[tokio::test]
async fn publish_rejects_system_topic() {
    let (bus, _dir) = fresh_bus().await;
    let err = bus
        .publish(NewMessage {
            plan_id: "plan-1".into(),
            topic: "system.session-events".into(),
            sender: None,
            payload: json!({}),
        })
        .await
        .unwrap_err();
    assert!(
        matches!(err, BusError::InvalidTopicScope(_)),
        "publish must refuse system.* topics: got {err:?}"
    );
}

#[tokio::test]
async fn publish_system_rejects_non_system_topic() {
    let (bus, _dir) = fresh_bus().await;
    let err = bus
        .publish_system(NewSystemMessage {
            topic: "task.done".into(),
            sender: None,
            payload: json!({}),
        })
        .await
        .unwrap_err();
    assert!(
        matches!(err, BusError::InvalidTopicScope(_)),
        "publish_system must refuse non-system.* topics: got {err:?}"
    );
}

#[tokio::test]
async fn poll_rejects_system_topic() {
    let (bus, _dir) = fresh_bus().await;
    let err = bus
        .poll("plan-1", "system.session-events", 0, 10)
        .await
        .unwrap_err();
    assert!(matches!(err, BusError::InvalidTopicScope(_)));
}

#[tokio::test]
async fn poll_system_rejects_non_system_topic() {
    let (bus, _dir) = fresh_bus().await;
    let err = bus.poll_system("task.done", 0, 10).await.unwrap_err();
    assert!(matches!(err, BusError::InvalidTopicScope(_)));
}

#[tokio::test]
async fn system_and_plan_traffic_are_isolated() {
    let (bus, _dir) = fresh_bus().await;

    bus.publish(NewMessage {
        plan_id: "plan-1".into(),
        topic: "task.done".into(),
        sender: None,
        payload: json!({"a": 1}),
    })
    .await
    .unwrap();

    bus.publish_system(NewSystemMessage {
        topic: "system.session-events".into(),
        sender: None,
        payload: json!({"a": 2}),
    })
    .await
    .unwrap();

    let plan_traffic = bus.poll("plan-1", "task.done", 0, 10).await.unwrap();
    assert_eq!(plan_traffic.len(), 1);
    assert_eq!(plan_traffic[0].plan_id.as_deref(), Some("plan-1"));

    let system_traffic = bus
        .poll_system("system.session-events", 0, 10)
        .await
        .unwrap();
    assert_eq!(system_traffic.len(), 1);
    assert!(system_traffic[0].plan_id.is_none());
}

#[tokio::test]
async fn check_constraint_rejects_null_plan_id_for_non_system_topic() {
    // Defence-in-depth: even if a future bug skipped Bus::publish, the
    // CHECK constraint at the SQL layer must still refuse a NULL
    // plan_id paired with a non-`system.` topic.
    let (bus, _dir) = fresh_bus().await;

    // Take a row in via system topic to confirm baseline works.
    bus.publish_system(NewSystemMessage {
        topic: "system.session-events".into(),
        sender: None,
        payload: json!({}),
    })
    .await
    .unwrap();

    // Now try to bypass via raw SQL — should fail the CHECK.
    let pool = Pool::connect(&format!("sqlite://{}/state.db", _dir.path().display()))
        .await
        .unwrap();
    let result = sqlx::query(
        "INSERT INTO agent_messages \
         (id, seq, plan_id, topic, sender, payload, consumed_at, consumed_by, created_at) \
         VALUES ('rogue', 9999, NULL, 'task.rogue', NULL, '{}', NULL, NULL, '2026-05-01T00:00:00Z')",
    )
    .execute(pool.inner())
    .await;
    assert!(
        result.is_err(),
        "CHECK constraint must refuse plan_id NULL paired with non-system topic"
    );
}
