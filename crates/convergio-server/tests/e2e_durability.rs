//! Workspace-level end-to-end test.
//!
//! Boots the convergio-server router in-process against a tempdir
//! SQLite, drives the full lifecycle of a plan + task + evidence over
//! HTTP, and verifies the audit chain.

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
async fn full_lifecycle_with_audit_verification() {
    let (base, _dir) = boot().await;
    let client = reqwest::Client::new();

    // Health probe
    let health: Value = client
        .get(format!("{base}/v1/health"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(health["ok"], true);

    // Create plan
    let plan: Value = client
        .post(format!("{base}/v1/plans"))
        .json(&json!({"title": "e2e plan"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let plan_id = plan["id"].as_str().unwrap().to_string();
    assert_eq!(plan["status"], "draft");
    assert_eq!(plan["title"], "e2e plan");

    // Create task with one required evidence kind
    let task: Value = client
        .post(format!("{base}/v1/plans/{plan_id}/tasks"))
        .json(&json!({
            "title": "ship feature",
            "evidence_required": ["test_pass"],
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let task_id = task["id"].as_str().unwrap().to_string();
    assert_eq!(task["status"], "pending");

    // Move to in_progress (no evidence required for this transition)
    let task: Value = client
        .post(format!("{base}/v1/tasks/{task_id}/transition"))
        .json(&json!({"target": "in_progress", "agent_id": "agent-1"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(task["status"], "in_progress");
    assert_eq!(task["agent_id"], "agent-1");

    // Try to submit without evidence — gate should refuse with 409
    let resp = client
        .post(format!("{base}/v1/tasks/{task_id}/transition"))
        .json(&json!({"target": "submitted"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409, "evidence gate should refuse");
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "gate_refused");

    // Attach the required evidence
    let _: Value = client
        .post(format!("{base}/v1/tasks/{task_id}/evidence"))
        .json(&json!({
            "kind": "test_pass",
            "payload": {"output": "1 passed"},
            "exit_code": 0,
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // Now the gate should allow submitted
    let task: Value = client
        .post(format!("{base}/v1/tasks/{task_id}/transition"))
        .json(&json!({"target": "submitted"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(task["status"], "submitted");

    // Audit chain must verify
    let report: Value = client
        .get(format!("{base}/v1/audit/verify"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(report["ok"], true, "audit chain should verify: {report}");
    assert!(
        report["checked"].as_i64().unwrap() >= 5,
        "expected 5+ events: plan.created, task.created, task.in_progress, evidence.attached, task.submitted"
    );
    assert_eq!(report["broken_at"], Value::Null);
}
