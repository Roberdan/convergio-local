//! Agent registry tests.

use convergio_db::Pool;
use convergio_durability::{init, AgentHeartbeat, Durability, DurabilityError, NewAgent};
use serde_json::json;
use tempfile::TempDir;

async fn fresh() -> (Durability, TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("state.db");
    let pool = Pool::connect(&format!("sqlite://{}", db.display()))
        .await
        .unwrap();
    init(&pool).await.unwrap();
    (Durability::new(pool), dir)
}

fn new_agent(id: &str) -> NewAgent {
    NewAgent {
        id: id.into(),
        kind: "copilot".into(),
        name: Some("Copilot worker".into()),
        host: Some("terminal".into()),
        capabilities: vec!["code".into(), "test".into()],
        metadata: json!({"pid": 123}),
    }
}

#[tokio::test]
async fn register_is_idempotent_and_heartbeat_updates_status() {
    let (dur, _dir) = fresh().await;
    let agent = dur.register_agent(new_agent("agent-a")).await.unwrap();
    assert_eq!(agent.status, "idle");
    assert_eq!(agent.capabilities, ["code", "test"]);

    let agent = dur.register_agent(new_agent("agent-a")).await.unwrap();
    assert_eq!(agent.id, "agent-a");
    let agent = dur
        .heartbeat_agent(
            "agent-a",
            AgentHeartbeat {
                current_task_id: Some("task-1".into()),
                status: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(agent.status, "working");
    assert_eq!(agent.current_task_id.as_deref(), Some("task-1"));

    let listed = dur.agents().list().await.unwrap();
    assert_eq!(listed.len(), 1);
    assert!(dur.audit().verify(None, None).await.unwrap().ok);
}

#[tokio::test]
async fn invalid_and_unknown_agents_are_rejected() {
    let (dur, _dir) = fresh().await;
    let err = dur.register_agent(new_agent("bad id")).await.unwrap_err();
    assert!(matches!(err, DurabilityError::InvalidAgent { .. }));

    let err = dur
        .heartbeat_agent(
            "missing",
            AgentHeartbeat {
                current_task_id: None,
                status: None,
            },
        )
        .await
        .unwrap_err();
    assert!(matches!(err, DurabilityError::NotFound { entity, .. } if entity == "agent"));
}

#[tokio::test]
async fn retire_marks_agent_terminated() {
    let (dur, _dir) = fresh().await;
    dur.register_agent(new_agent("agent-a")).await.unwrap();
    let agent = dur.retire_agent("agent-a").await.unwrap();
    assert_eq!(agent.status, "terminated");
    assert!(agent.current_task_id.is_none());
}
