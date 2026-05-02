//! Executor integration tests.

use chrono::Duration as ChronoDuration;
use convergio_db::Pool;
use convergio_durability::{init, Durability, TaskStatus};
use convergio_executor::{spawn_loop, Executor, SpawnTemplate};
use convergio_lifecycle::Supervisor;
use convergio_planner::Planner;
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

async fn fresh() -> (Executor, Durability, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let url = format!("sqlite://{}/state.db", dir.path().display());
    let pool = Pool::connect(&url).await.unwrap();
    init(&pool).await.unwrap();
    convergio_lifecycle::init(&pool).await.unwrap();
    let dur = Durability::new(pool.clone());
    let sup = Supervisor::new(pool);
    let exec = Executor::new(dur.clone(), sup, SpawnTemplate::default());
    (exec, dur, dir)
}

#[tokio::test]
async fn tick_dispatches_pending_tasks_in_first_wave() {
    let (exec, dur, _dir) = fresh().await;
    let planner = Planner::new(dur.clone());
    let plan_id = planner.solve("a\nb\nc").await.unwrap();

    let dispatched = exec.tick().await.unwrap();
    assert_eq!(dispatched, 3);

    let tasks = dur.tasks().list_by_plan(&plan_id).await.unwrap();
    assert!(tasks.iter().all(|t| t.status == TaskStatus::InProgress));
    assert!(tasks.iter().all(|t| t.agent_id.is_some()));
}

#[tokio::test]
async fn tick_skips_later_waves_until_earlier_done() {
    let (exec, dur, _dir) = fresh().await;
    let plan = dur
        .create_plan(convergio_durability::NewPlan {
            title: "p".into(),
            description: None,
            project: None,
        })
        .await
        .unwrap();
    let _w1 = dur
        .create_task(
            &plan.id,
            convergio_durability::NewTask {
                wave: 1,
                sequence: 1,
                title: "wave1".into(),
                description: None,
                evidence_required: vec![],
            },
        )
        .await
        .unwrap();
    let w2 = dur
        .create_task(
            &plan.id,
            convergio_durability::NewTask {
                wave: 2,
                sequence: 1,
                title: "wave2".into(),
                description: None,
                evidence_required: vec![],
            },
        )
        .await
        .unwrap();

    // First tick: only wave 1 dispatched.
    let n = exec.tick().await.unwrap();
    assert_eq!(n, 1);
    let after = dur.tasks().get(&w2.id).await.unwrap();
    assert_eq!(after.status, TaskStatus::Pending);

    // Second tick: wave 1 still in_progress so wave 2 still waits.
    let n = exec.tick().await.unwrap();
    assert_eq!(n, 0);
}

#[tokio::test]
async fn tick_is_idempotent_on_already_dispatched_tasks() {
    let (exec, dur, _dir) = fresh().await;
    let planner = Planner::new(dur.clone());
    planner.solve("only one").await.unwrap();

    let n1 = exec.tick().await.unwrap();
    let n2 = exec.tick().await.unwrap();
    assert_eq!(n1, 1);
    assert_eq!(n2, 0);
}

#[tokio::test]
async fn dispatch_writes_audit_chain_that_verifies() {
    let (exec, dur, _dir) = fresh().await;
    let planner = Planner::new(dur.clone());
    planner.solve("x\ny").await.unwrap();
    exec.tick().await.unwrap();

    let r = dur.audit().verify(None, None).await.unwrap();
    assert!(r.ok, "{r:?}");
    // 1 plan.created + 2 task.created + 2 task.in_progress = 5+
    assert!(r.checked >= 5);
}

#[tokio::test]
async fn spawn_loop_dispatches_pending_tasks_in_background() {
    // Wires the same loop the daemon's main.rs runs (ADR-0027). A
    // pending task with no wave dependencies must be promoted to
    // in_progress within one tick of the loop, with no manual
    // `Executor::tick()` or `POST /v1/dispatch` call.
    let (exec, dur, _dir) = fresh().await;
    let planner = Planner::new(dur.clone());
    let plan_id = planner.solve("loop-task").await.unwrap();

    let handle = spawn_loop(Arc::new(exec), ChronoDuration::milliseconds(50));

    // Poll up to 5 seconds for the loop to flip the task. With a 50ms
    // tick and a single-task plan, the first round should land in
    // ~50-100ms; the budget is wide so this stays green on slow CI.
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let mut promoted = false;
    while std::time::Instant::now() < deadline {
        let tasks = dur.tasks().list_by_plan(&plan_id).await.unwrap();
        if tasks.iter().all(|t| t.status == TaskStatus::InProgress) {
            promoted = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    handle.abort();
    assert!(promoted, "spawn_loop did not dispatch within 5s");
}
