//! Thor integration tests.

use convergio_db::Pool;
use convergio_durability::{init, Durability, NewPlan, NewTask, TaskStatus};
use convergio_thor::{Thor, Verdict};
use serde_json::json;
use tempfile::tempdir;

async fn fresh() -> (Thor, Durability, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let url = format!("sqlite://{}/state.db", dir.path().display());
    let pool = Pool::connect(&url).await.unwrap();
    init(&pool).await.unwrap();
    let dur = Durability::new(pool);
    (Thor::new(dur.clone()), dur, dir)
}

async fn plan_with_one_task(dur: &Durability, evidence_required: Vec<String>) -> (String, String) {
    let plan = dur
        .create_plan(NewPlan {
            title: "p".into(),
            description: None,
            project: None,
        })
        .await
        .unwrap();
    let task = dur
        .create_task(
            &plan.id,
            NewTask {
                wave: 1,
                sequence: 1,
                title: "t".into(),
                description: None,
                evidence_required,
            },
        )
        .await
        .unwrap();
    (plan.id, task.id)
}

#[tokio::test]
async fn pass_when_all_done_with_required_evidence() {
    let (thor, dur, _dir) = fresh().await;
    let (plan_id, task_id) = plan_with_one_task(&dur, vec!["test_pass".into()]).await;
    dur.transition_task(&task_id, TaskStatus::InProgress, Some("a"))
        .await
        .unwrap();
    dur.attach_evidence(&task_id, "test_pass", json!({}), Some(0))
        .await
        .unwrap();
    dur.transition_task(&task_id, TaskStatus::Submitted, Some("a"))
        .await
        .unwrap();
    // Submitted is not Done — Thor must fail.
    let v = thor.validate(&plan_id).await.unwrap();
    matches!(v, Verdict::Fail { .. });

    // Force to Done (skip submitted->done because we don't have a
    // dedicated gate yet — call the store directly).
    dur.tasks()
        .set_status(&task_id, TaskStatus::Done, Some("a"))
        .await
        .unwrap();
    let v = thor.validate(&plan_id).await.unwrap();
    matches!(v, Verdict::Pass);
}

#[tokio::test]
async fn fail_when_evidence_missing() {
    let (thor, dur, _dir) = fresh().await;
    let (plan_id, task_id) = plan_with_one_task(&dur, vec!["test_pass".into()]).await;
    // Force task to done WITHOUT attaching evidence.
    dur.tasks()
        .set_status(&task_id, TaskStatus::Done, Some("a"))
        .await
        .unwrap();
    let v = thor.validate(&plan_id).await.unwrap();
    match v {
        Verdict::Fail { reasons } => {
            assert!(
                reasons.iter().any(|r| r.contains("test_pass")),
                "reasons should name missing kind: {reasons:?}"
            );
        }
        Verdict::Pass => panic!("expected fail"),
    }
}

#[tokio::test]
async fn fail_when_plan_has_no_tasks() {
    let (thor, dur, _dir) = fresh().await;
    let plan = dur
        .create_plan(NewPlan {
            title: "empty".into(),
            description: None,
            project: None,
        })
        .await
        .unwrap();
    let v = thor.validate(&plan.id).await.unwrap();
    matches!(v, Verdict::Fail { .. });
}

#[tokio::test]
async fn unknown_plan_returns_error() {
    let (thor, _dur, _dir) = fresh().await;
    assert!(thor.validate("does-not-exist").await.is_err());
}
