//! ADR-0024 — `Bus::poll_filtered` with `exclude_sender`.

use convergio_bus::{init, Bus, NewMessage};
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
async fn poll_filtered_skips_sender_match() {
    let (bus, _dir) = fresh_bus().await;
    bus.publish(NewMessage {
        plan_id: "plan-1".into(),
        topic: "coord".into(),
        sender: Some("agent-a".into()),
        payload: json!({"from": "a"}),
    })
    .await
    .unwrap();
    bus.publish(NewMessage {
        plan_id: "plan-1".into(),
        topic: "coord".into(),
        sender: Some("agent-b".into()),
        payload: json!({"from": "b"}),
    })
    .await
    .unwrap();

    let plain = bus.poll("plan-1", "coord", 0, 10).await.unwrap();
    assert_eq!(plain.len(), 2);

    let filtered = bus
        .poll_filtered("plan-1", "coord", 0, 10, Some("agent-a"))
        .await
        .unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].sender.as_deref(), Some("agent-b"));
}

#[tokio::test]
async fn poll_filtered_keeps_null_sender_system_messages() {
    let (bus, _dir) = fresh_bus().await;
    bus.publish(NewMessage {
        plan_id: "plan-1".into(),
        topic: "events".into(),
        sender: None,
        payload: json!({"kind": "system"}),
    })
    .await
    .unwrap();
    bus.publish(NewMessage {
        plan_id: "plan-1".into(),
        topic: "events".into(),
        sender: Some("agent-a".into()),
        payload: json!({"from": "a"}),
    })
    .await
    .unwrap();

    let filtered = bus
        .poll_filtered("plan-1", "events", 0, 10, Some("agent-a"))
        .await
        .unwrap();
    // Excludes agent-a, keeps the null-sender system row.
    assert_eq!(filtered.len(), 1);
    assert!(filtered[0].sender.is_none());
}

#[tokio::test]
async fn poll_filtered_with_none_matches_plain_poll() {
    let (bus, _dir) = fresh_bus().await;
    bus.publish(NewMessage {
        plan_id: "plan-1".into(),
        topic: "topic".into(),
        sender: Some("a".into()),
        payload: json!({}),
    })
    .await
    .unwrap();
    let p1 = bus.poll("plan-1", "topic", 0, 10).await.unwrap();
    let p2 = bus
        .poll_filtered("plan-1", "topic", 0, 10, None)
        .await
        .unwrap();
    assert_eq!(p1.len(), p2.len());
    assert_eq!(p1[0].id, p2[0].id);
}
