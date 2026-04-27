//! Layer 3 integration tests — spawn `/bin/echo` and friends.

use convergio_db::Pool;
use convergio_lifecycle::{init, ProcessStatus, SpawnSpec, Supervisor};
use tempfile::tempdir;

async fn fresh_supervisor() -> (Supervisor, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let url = format!("sqlite://{}/state.db", dir.path().display());
    let pool = Pool::connect(&url).await.unwrap();
    init(&pool).await.unwrap();
    (Supervisor::new(pool), dir)
}

#[tokio::test]
async fn spawn_records_running_process_with_pid() {
    let (sup, _dir) = fresh_supervisor().await;

    let proc = sup
        .spawn(SpawnSpec {
            kind: "shell".into(),
            command: "/bin/echo".into(),
            args: vec!["hello-from-test".into()],
            env: vec![],
            plan_id: Some("plan-1".into()),
            task_id: None,
        })
        .await
        .unwrap();

    assert_eq!(proc.kind, "shell");
    assert_eq!(proc.status, ProcessStatus::Running);
    assert!(proc.pid.is_some());

    // Round-trip via the DB.
    let same = sup.get(&proc.id).await.unwrap();
    assert_eq!(same.id, proc.id);
    assert_eq!(same.command, "/bin/echo");
}

#[tokio::test]
async fn spawn_failure_returns_error() {
    let (sup, _dir) = fresh_supervisor().await;
    let err = sup
        .spawn(SpawnSpec {
            kind: "shell".into(),
            command: "/no/such/binary/please".into(),
            args: vec![],
            env: vec![],
            plan_id: None,
            task_id: None,
        })
        .await
        .unwrap_err();
    matches!(err, convergio_lifecycle::LifecycleError::SpawnFailed(_));
}

#[tokio::test]
async fn heartbeat_touches_row() {
    let (sup, _dir) = fresh_supervisor().await;
    let proc = sup
        .spawn(SpawnSpec {
            kind: "shell".into(),
            command: "/bin/echo".into(),
            args: vec!["hb".into()],
            env: vec![],
            plan_id: None,
            task_id: None,
        })
        .await
        .unwrap();

    sup.heartbeat(&proc.id).await.unwrap();
    let after = sup.get(&proc.id).await.unwrap();
    assert!(after.last_heartbeat_at.is_some());

    // Unknown id is NotFound.
    let err = sup.heartbeat("nope").await.unwrap_err();
    matches!(err, convergio_lifecycle::LifecycleError::NotFound(_));
}

#[tokio::test]
async fn mark_exited_records_exit_code() {
    let (sup, _dir) = fresh_supervisor().await;
    let proc = sup
        .spawn(SpawnSpec {
            kind: "shell".into(),
            command: "/bin/echo".into(),
            args: vec!["bye".into()],
            env: vec![],
            plan_id: None,
            task_id: None,
        })
        .await
        .unwrap();

    sup.mark_exited(&proc.id, Some(0), true).await.unwrap();
    let after = sup.get(&proc.id).await.unwrap();
    assert_eq!(after.status, ProcessStatus::Exited);
    assert_eq!(after.exit_code, Some(0));
    assert!(after.ended_at.is_some());
}
