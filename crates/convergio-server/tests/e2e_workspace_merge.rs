//! Workspace merge queue E2E tests.

use chrono::{Duration, Utc};
use convergio_bus::Bus;
use convergio_db::Pool;
use convergio_durability::init;
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
    let pool = Pool::connect(&format!("sqlite://{}", db_path.display()))
        .await
        .unwrap();
    init(&pool).await.unwrap();
    convergio_bus::init(&pool).await.unwrap();
    convergio_lifecycle::init(&pool).await.unwrap();
    let state = AppState {
        durability: Arc::new(convergio_durability::Durability::new(pool.clone())),
        bus: Arc::new(Bus::new(pool.clone())),
        supervisor: Arc::new(Supervisor::new(pool)),
    };
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router(state)).await.unwrap();
    });
    (format!("http://{addr}"), dir)
}

fn claim(agent_id: &str, path: &str) -> Value {
    json!({
        "resource": {"kind": "file", "path": path},
        "task_id": "task-lease",
        "agent_id": agent_id,
        "purpose": "edit",
        "expires_at": (Utc::now() + Duration::minutes(10)).to_rfc3339()
    })
}

fn patch(agent_id: &str, path: &str, proposed_hash: &str) -> Value {
    json!({
        "task_id": "task-lease",
        "agent_id": agent_id,
        "base_revision": "base",
        "patch": "diff --git",
        "files": [{
            "path": path,
            "project": null,
            "base_hash": "same",
            "current_hash": "same",
            "proposed_hash": proposed_hash
        }]
    })
}

async fn submit_enqueue_process(
    client: &reqwest::Client,
    base: &str,
    body: Value,
) -> reqwest::Response {
    let proposal: Value = client
        .post(format!("{base}/v1/workspace/patches"))
        .json(&body)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let proposal_id = proposal["id"].as_str().unwrap();
    let _: Value = client
        .post(format!("{base}/v1/workspace/patches/{proposal_id}/enqueue"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    client
        .post(format!("{base}/v1/workspace/merge/next"))
        .send()
        .await
        .unwrap()
}

#[tokio::test]
async fn merge_queue_merges_different_files_and_refuses_stale_same_file() {
    let (base, _dir) = boot().await;
    let client = reqwest::Client::new();
    for (agent, path) in [("agent-a", "src/lib.rs"), ("agent-b", "src/main.rs")] {
        let _: Value = client
            .post(format!("{base}/v1/workspace/leases"))
            .json(&claim(agent, path))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
    }

    let resp = submit_enqueue_process(&client, &base, patch("agent-a", "src/lib.rs", "next")).await;
    assert_eq!(resp.status(), 200);
    let merged: Value = resp.json().await.unwrap();
    assert_eq!(merged["item"]["status"], "merged");

    let resp =
        submit_enqueue_process(&client, &base, patch("agent-b", "src/main.rs", "main-next")).await;
    assert_eq!(resp.status(), 200);

    let resp =
        submit_enqueue_process(&client, &base, patch("agent-a", "src/lib.rs", "other")).await;
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "workspace_merge_refused");
}
