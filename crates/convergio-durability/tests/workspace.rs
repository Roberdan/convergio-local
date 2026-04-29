//! Workspace resource/lease store tests.

use chrono::{Duration, Utc};
use convergio_db::Pool;
use convergio_durability::{
    init, Durability, DurabilityError, NewWorkspaceLease, NewWorkspaceResource,
};
use tempfile::tempdir;

async fn fresh() -> (Durability, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let url = format!("sqlite://{}/state.db", dir.path().display());
    let pool = Pool::connect(&url).await.unwrap();
    init(&pool).await.unwrap();
    (Durability::new(pool), dir)
}

fn resource(path: &str) -> NewWorkspaceResource {
    NewWorkspaceResource {
        kind: "file".into(),
        project: Some("convergio-local".into()),
        path: path.into(),
        symbol: None,
    }
}

fn lease(agent_id: &str, path: &str) -> NewWorkspaceLease {
    NewWorkspaceLease {
        resource: resource(path),
        task_id: Some("task-1".into()),
        agent_id: agent_id.into(),
        purpose: Some("edit".into()),
        expires_at: Utc::now() + Duration::minutes(10),
    }
}

#[tokio::test]
async fn claiming_same_resource_conflicts_until_release() {
    let (dur, _dir) = fresh().await;

    let first = dur
        .workspace()
        .claim_lease(lease("agent-a", "src/lib.rs"))
        .await
        .unwrap();
    let err = dur
        .workspace()
        .claim_lease(lease("agent-b", "src/lib.rs"))
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        DurabilityError::WorkspaceLeaseConflict {
            resource_id: _,
            lease_id,
            agent_id,
        } if lease_id == first.id && agent_id == "agent-a"
    ));

    let released = dur.workspace().release_lease(&first.id).await.unwrap();
    assert_eq!(released.status, "released");

    let second = dur
        .workspace()
        .claim_lease(lease("agent-b", "src/lib.rs"))
        .await
        .unwrap();
    assert_eq!(second.agent_id, "agent-b");
}

#[tokio::test]
async fn expired_lease_does_not_block_new_claim() {
    let (dur, _dir) = fresh().await;
    let first = dur
        .workspace()
        .claim_lease(lease("agent-a", "src/main.rs"))
        .await
        .unwrap();

    let expired = dur
        .workspace()
        .expire_leases(Utc::now() + Duration::hours(1))
        .await
        .unwrap();
    assert_eq!(expired, 1);

    let second = dur
        .workspace()
        .claim_lease(lease("agent-b", "src/main.rs"))
        .await
        .unwrap();

    assert_ne!(first.id, second.id);
    assert_eq!(second.status, "active");
}

#[tokio::test]
async fn active_lease_list_excludes_released_rows() {
    let (dur, _dir) = fresh().await;
    let first = dur
        .workspace()
        .claim_lease(lease("agent-a", "README.md"))
        .await
        .unwrap();
    dur.workspace().release_lease(&first.id).await.unwrap();

    let active = dur.workspace().active_leases().await.unwrap();
    assert!(active.is_empty());
}
