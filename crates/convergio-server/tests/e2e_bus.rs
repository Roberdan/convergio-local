//! HTTP end-to-end test for Layer 2 (`convergio-bus`).

use convergio_bus::Bus;
use convergio_db::Pool;
use convergio_durability::{init, Durability};
use convergio_lifecycle::Supervisor;
use convergio_server::{router, AppState};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::net::TcpListener;

async fn boot() -> (String, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("state.db");
    let url = format!("sqlite://{}", db_path.display());
    let pool = Pool::connect(&url).await.unwrap();
    init(&pool).await.unwrap();
    convergio_bus::init(&pool).await.unwrap();
    convergio_lifecycle::init(&pool).await.unwrap();

    let state = AppState {
        durability: Arc::new(Durability::new(pool.clone())),
        bus: Arc::new(Bus::new(pool.clone())),
        supervisor: Arc::new(Supervisor::new(pool)),
    };
    let app = router(state);

    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}"), dir)
}

#[tokio::test]
async fn publish_poll_ack_round_trip_over_http() {
    let (base, _dir) = boot().await;
    let client = reqwest::Client::new();
    let plan_id = "plan-x";

    // Publish two messages.
    for i in 0..2 {
        let _: Value = client
            .post(format!("{base}/v1/plans/{plan_id}/messages"))
            .json(&json!({
                "topic": "task.done",
                "sender": "agent-1",
                "payload": {"i": i},
            }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
    }

    // Poll.
    let messages: Vec<Value> = client
        .get(format!(
            "{base}/v1/plans/{plan_id}/messages?topic=task.done&limit=10"
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0]["payload"]["i"], 0);
    assert_eq!(messages[1]["payload"]["i"], 1);

    // Ack the first one.
    let id = messages[0]["id"].as_str().unwrap();
    let _: Value = client
        .post(format!("{base}/v1/messages/{id}/ack"))
        .json(&json!({"consumer": "agent-2"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // Poll again — only the second should remain.
    let messages: Vec<Value> = client
        .get(format!(
            "{base}/v1/plans/{plan_id}/messages?topic=task.done&limit=10"
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["payload"]["i"], 1);
}

#[tokio::test]
async fn ack_unknown_returns_404() {
    let (base, _dir) = boot().await;
    let resp = reqwest::Client::new()
        .post(format!("{base}/v1/messages/no-such-id/ack"))
        .json(&json!({}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "not_found");
}

#[tokio::test]
async fn poll_rejects_unbounded_limit() {
    let (base, _dir) = boot().await;
    let resp = reqwest::Client::new()
        .get(format!("{base}/v1/plans/plan-x/messages?topic=t&limit=101"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "invalid_message_limit");
}
