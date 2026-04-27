//! HTTP end-to-end test for Layer 3 (`convergio-lifecycle`).

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
async fn spawn_get_heartbeat_round_trip_over_http() {
    let (base, _dir) = boot().await;
    let client = reqwest::Client::new();

    let proc: Value = client
        .post(format!("{base}/v1/agents/spawn"))
        .json(&json!({
            "kind": "shell",
            "command": "/bin/echo",
            "args": ["e2e"],
            "env": [],
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id = proc["id"].as_str().unwrap();
    assert_eq!(proc["kind"], "shell");
    assert_eq!(proc["status"], "running");
    assert!(proc["pid"].is_number());

    // GET should return the same row.
    let fetched: Value = client
        .get(format!("{base}/v1/agents/{id}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(fetched["id"], id);

    // Heartbeat ok.
    let hb: Value = client
        .post(format!("{base}/v1/agents/{id}/heartbeat"))
        .json(&json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(hb["ok"], true);
}

#[tokio::test]
async fn spawn_invalid_command_returns_422() {
    let (base, _dir) = boot().await;
    let resp = reqwest::Client::new()
        .post(format!("{base}/v1/agents/spawn"))
        .json(&json!({
            "kind": "shell",
            "command": "/no/such/binary/anywhere",
            "args": [],
            "env": [],
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 422);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "spawn_failed");
}
