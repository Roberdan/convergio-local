//! Reaper integration test — drives `tick` directly so it doesn't have
//! to wait on real wall-clock time.

use chrono::{Duration, Utc};
use convergio_db::Pool;
use convergio_durability::reaper::{self, ReaperConfig};
use convergio_durability::{init, Durability, NewPlan, NewTask, TaskStatus};
use tempfile::tempdir;

async fn fresh_durability() -> (Durability, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let url = format!("sqlite://{}/state.db", dir.path().display());
    let pool = Pool::connect(&url).await.unwrap();
    init(&pool).await.unwrap();
    (Durability::new(pool), dir)
}

#[tokio::test]
async fn reaps_tasks_with_stale_heartbeat() {
    let (dur, _dir) = fresh_durability().await;

    let plan = dur
        .create_plan(NewPlan {
            title: "reaper test".into(),
            description: None,
        })
        .await
        .unwrap();

    let task = dur
        .create_task(
            &plan.id,
            NewTask {
                wave: 1,
                sequence: 1,
                title: "stuck task".into(),
                description: None,
                evidence_required: vec![],
            },
        )
        .await
        .unwrap();

    // Claim it as agent-1
    dur.transition_task(&task.id, TaskStatus::InProgress, Some("agent-1"))
        .await
        .unwrap();

    // Forge a stale heartbeat by writing it back-dated.
    let stale = (Utc::now() - Duration::seconds(3600)).to_rfc3339();
    sqlx::query("UPDATE tasks SET last_heartbeat_at = ? WHERE id = ?")
        .bind(&stale)
        .bind(&task.id)
        .execute(dur.pool().inner())
        .await
        .unwrap();

    // Tick with a 5-minute timeout — task is 1h stale, must be reaped.
    let n = reaper::tick(
        &dur,
        &ReaperConfig {
            timeout: Duration::seconds(300),
            tick_interval: Duration::seconds(60),
        },
    )
    .await
    .unwrap();
    assert_eq!(n, 1);

    let after = dur.tasks().get(&task.id).await.unwrap();
    assert_eq!(after.status, TaskStatus::Pending);
    assert!(after.agent_id.is_none());

    // Audit chain must include task.reaped and still verify clean.
    let report = dur.audit().verify(None, None).await.unwrap();
    assert!(report.ok);
}

#[tokio::test]
async fn does_not_reap_fresh_tasks() {
    let (dur, _dir) = fresh_durability().await;

    let plan = dur
        .create_plan(NewPlan {
            title: "fresh".into(),
            description: None,
        })
        .await
        .unwrap();
    let task = dur
        .create_task(
            &plan.id,
            NewTask {
                wave: 1,
                sequence: 1,
                title: "fresh task".into(),
                description: None,
                evidence_required: vec![],
            },
        )
        .await
        .unwrap();
    dur.transition_task(&task.id, TaskStatus::InProgress, Some("agent-1"))
        .await
        .unwrap();
    dur.tasks().heartbeat(&task.id).await.unwrap();

    let n = reaper::tick(
        &dur,
        &ReaperConfig {
            timeout: Duration::seconds(60),
            tick_interval: Duration::seconds(30),
        },
    )
    .await
    .unwrap();
    assert_eq!(n, 0);

    let after = dur.tasks().get(&task.id).await.unwrap();
    assert_eq!(after.status, TaskStatus::InProgress);
}

#[tokio::test]
async fn reaps_tasks_that_never_heartbeat() {
    let (dur, _dir) = fresh_durability().await;

    let plan = dur
        .create_plan(NewPlan {
            title: "never heartbeat".into(),
            description: None,
        })
        .await
        .unwrap();
    let task = dur
        .create_task(
            &plan.id,
            NewTask {
                wave: 1,
                sequence: 1,
                title: "claimed then died".into(),
                description: None,
                evidence_required: vec![],
            },
        )
        .await
        .unwrap();
    dur.transition_task(&task.id, TaskStatus::InProgress, Some("agent-1"))
        .await
        .unwrap();

    let stale = (Utc::now() - Duration::seconds(3600)).to_rfc3339();
    sqlx::query("UPDATE tasks SET updated_at = ?, last_heartbeat_at = NULL WHERE id = ?")
        .bind(&stale)
        .bind(&task.id)
        .execute(dur.pool().inner())
        .await
        .unwrap();

    let n = reaper::tick(
        &dur,
        &ReaperConfig {
            timeout: Duration::seconds(300),
            tick_interval: Duration::seconds(60),
        },
    )
    .await
    .unwrap();
    assert_eq!(n, 1);

    let after = dur.tasks().get(&task.id).await.unwrap();
    assert_eq!(after.status, TaskStatus::Pending);
    assert!(after.agent_id.is_none());
}
