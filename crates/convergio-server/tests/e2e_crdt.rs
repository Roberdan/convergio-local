//! CRDT diagnostics and conflict-gate E2E tests.

use convergio_bus::Bus;
use convergio_db::Pool;
use convergio_durability::{init, Durability, NewCrdtOp};
use convergio_lifecycle::Supervisor;
use convergio_server::{router, AppState};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::net::TcpListener;

async fn boot() -> (String, Arc<Durability>, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("state.db");
    let url = format!("sqlite://{}", db_path.display());
    let pool = Pool::connect(&url).await.unwrap();
    init(&pool).await.unwrap();
    convergio_bus::init(&pool).await.unwrap();
    convergio_lifecycle::init(&pool).await.unwrap();

    let durability = Arc::new(Durability::new(pool.clone()));
    let state = AppState {
        durability: durability.clone(),
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
    (format!("http://{addr}"), durability, dir)
}

fn op(actor_id: &str, task_id: &str, value: &str) -> NewCrdtOp {
    NewCrdtOp {
        actor_id: actor_id.to_string(),
        counter: 1,
        entity_type: "task".into(),
        entity_id: task_id.into(),
        field_name: "title".into(),
        crdt_type: "mv_register".into(),
        op_kind: "set".into(),
        value: json!(value),
        hlc: "2026-04-29T00:00:00Z/0".into(),
    }
}

#[tokio::test]
async fn crdt_conflicts_are_listed_and_block_task_submission() {
    let (base, durability, _dir) = boot().await;
    let client = reqwest::Client::new();
    let plan: Value = client
        .post(format!("{base}/v1/plans"))
        .json(&json!({"title": "crdt api plan"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let plan_id = plan["id"].as_str().unwrap();
    let task: Value = client
        .post(format!("{base}/v1/plans/{plan_id}/tasks"))
        .json(&json!({"title": "conflicted task"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let task_id = task["id"].as_str().unwrap();

    durability
        .crdt()
        .append_op(op("actor-a", task_id, "alpha"))
        .await
        .unwrap();
    durability
        .crdt()
        .append_op(op("actor-b", task_id, "beta"))
        .await
        .unwrap();
    durability
        .crdt()
        .merge_cell("task", task_id, "title")
        .await
        .unwrap();

    let conflicts: Value = client
        .get(format!("{base}/v1/crdt/conflicts"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(conflicts.as_array().unwrap().len(), 1, "{conflicts}");
    assert_eq!(conflicts[0]["field_name"], "title");

    let resp = client
        .post(format!("{base}/v1/tasks/{task_id}/transition"))
        .json(&json!({"target": "submitted", "agent_id": "agent-crdt"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "gate_refused");
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("crdt_conflict"));
}
