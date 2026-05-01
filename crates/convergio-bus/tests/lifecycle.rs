//! Integration tests for the Layer 2 bus.

use convergio_bus::{init, Bus, NewMessage};
use convergio_db::Pool;
use serde_json::json;
use tempfile::tempdir;
use tokio::task::JoinSet;

async fn fresh_bus() -> (Bus, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let url = format!("sqlite://{}/state.db", dir.path().display());
    let pool = Pool::connect(&url).await.unwrap();
    init(&pool).await.unwrap();
    (Bus::new(pool), dir)
}

#[tokio::test]
async fn publish_then_poll_then_ack() {
    let (bus, _dir) = fresh_bus().await;

    let m = bus
        .publish(NewMessage {
            plan_id: "plan-1".into(),
            topic: "task.done".into(),
            sender: Some("agent-1".into()),
            payload: json!({"task_id": "t-1"}),
        })
        .await
        .unwrap();
    assert_eq!(m.seq, 1);
    assert!(m.consumed_at.is_none());

    let polled = bus.poll("plan-1", "task.done", 0, 10).await.unwrap();
    assert_eq!(polled.len(), 1);
    assert_eq!(polled[0].id, m.id);
    assert_eq!(polled[0].payload["task_id"], "t-1");

    bus.ack(&m.id, Some("agent-2")).await.unwrap();

    // After ack, poll yields nothing.
    let polled = bus.poll("plan-1", "task.done", 0, 10).await.unwrap();
    assert!(polled.is_empty());
}

#[tokio::test]
async fn fifo_per_topic_via_cursor() {
    let (bus, _dir) = fresh_bus().await;

    for i in 0..5 {
        bus.publish(NewMessage {
            plan_id: "plan-1".into(),
            topic: "events".into(),
            sender: None,
            payload: json!({"i": i}),
        })
        .await
        .unwrap();
    }

    let first_two = bus.poll("plan-1", "events", 0, 2).await.unwrap();
    assert_eq!(first_two.len(), 2);
    assert_eq!(first_two[0].payload["i"], 0);
    assert_eq!(first_two[1].payload["i"], 1);

    // Use the highest seq as the next cursor.
    let cursor = first_two[1].seq;
    let next = bus.poll("plan-1", "events", cursor, 10).await.unwrap();
    assert_eq!(next.len(), 3);
    assert_eq!(next[0].payload["i"], 2);
    assert_eq!(next[2].payload["i"], 4);
}

#[tokio::test]
async fn scope_per_plan() {
    let (bus, _dir) = fresh_bus().await;

    bus.publish(NewMessage {
        plan_id: "plan-1".into(),
        topic: "x".into(),
        sender: None,
        payload: json!({"who": "1"}),
    })
    .await
    .unwrap();
    bus.publish(NewMessage {
        plan_id: "plan-2".into(),
        topic: "x".into(),
        sender: None,
        payload: json!({"who": "2"}),
    })
    .await
    .unwrap();

    let p1 = bus.poll("plan-1", "x", 0, 10).await.unwrap();
    let p2 = bus.poll("plan-2", "x", 0, 10).await.unwrap();
    assert_eq!(p1.len(), 1);
    assert_eq!(p2.len(), 1);
    assert_eq!(p1[0].payload["who"], "1");
    assert_eq!(p2[0].payload["who"], "2");
}

#[tokio::test]
async fn ack_unknown_id_is_not_found() {
    let (bus, _dir) = fresh_bus().await;
    let err = bus.ack("does-not-exist", None).await.unwrap_err();
    matches!(err, convergio_bus::BusError::NotFound(_));
}

#[tokio::test]
async fn double_ack_is_idempotent() {
    let (bus, _dir) = fresh_bus().await;
    let m = bus
        .publish(NewMessage {
            plan_id: "p".into(),
            topic: "t".into(),
            sender: None,
            payload: json!({}),
        })
        .await
        .unwrap();
    bus.ack(&m.id, Some("c")).await.unwrap();
    // Re-acking does not error.
    bus.ack(&m.id, Some("c")).await.unwrap();
}

#[tokio::test]
async fn concurrent_publish_allocates_contiguous_sequences() {
    let (bus, _dir) = fresh_bus().await;
    let mut jobs = JoinSet::new();
    for i in 0..20 {
        let bus = bus.clone();
        jobs.spawn(async move {
            bus.publish(NewMessage {
                plan_id: "plan-1".into(),
                topic: "events".into(),
                sender: None,
                payload: json!({"i": i}),
            })
            .await
            .unwrap();
        });
    }
    while let Some(result) = jobs.join_next().await {
        result.unwrap();
    }

    let messages = bus.poll("plan-1", "events", 0, 100).await.unwrap();
    let seqs: Vec<i64> = messages.into_iter().map(|m| m.seq).collect();
    assert_eq!(seqs, (1..=20).collect::<Vec<_>>());
}
