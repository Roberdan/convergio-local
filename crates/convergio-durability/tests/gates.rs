//! Direct unit tests for each gate.
//!
//! The facade tests cover the happy path. These tests prove each
//! gate's refusal logic in isolation so a regression in one gate
//! cannot hide behind a passing E2E.

use convergio_db::Pool;
use convergio_durability::gates::{
    EvidenceGate, Gate, GateContext, PlanStatusGate, WaveSequenceGate,
};
use convergio_durability::{
    init, Durability, DurabilityError, NewPlan, NewTask, PlanStatus, TaskStatus,
};
use serde_json::json;
use tempfile::tempdir;

async fn fresh() -> (Durability, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let url = format!("sqlite://{}/state.db", dir.path().display());
    let pool: Pool = Pool::connect(&url).await.unwrap();
    init(&pool).await.unwrap();
    (Durability::new(pool), dir)
}

fn ctx(dur: &Durability, task: convergio_durability::Task, target: TaskStatus) -> GateContext {
    GateContext {
        pool: dur.pool().clone(),
        task,
        target_status: target,
        agent_id: Some("agent-1".into()),
    }
}

#[tokio::test]
async fn plan_status_gate_refuses_when_plan_completed() {
    let (dur, _dir) = fresh().await;
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
                evidence_required: vec![],
            },
        )
        .await
        .unwrap();

    // Force plan to completed.
    dur.plans()
        .set_status(&plan.id, PlanStatus::Completed)
        .await
        .unwrap();

    let gate = PlanStatusGate;
    let err = gate
        .check(&ctx(&dur, task, TaskStatus::InProgress))
        .await
        .unwrap_err();
    matches!(
        err,
        DurabilityError::GateRefused {
            gate: "plan_status",
            ..
        }
    );
}

#[tokio::test]
async fn plan_status_gate_passes_for_active_plan() {
    let (dur, _dir) = fresh().await;
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
                evidence_required: vec![],
            },
        )
        .await
        .unwrap();

    let gate = PlanStatusGate;
    gate.check(&ctx(&dur, task, TaskStatus::InProgress))
        .await
        .unwrap();
}

#[tokio::test]
async fn evidence_gate_refuses_when_kind_missing() {
    let (dur, _dir) = fresh().await;
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
                evidence_required: vec!["test_pass".into(), "pr_url".into()],
            },
        )
        .await
        .unwrap();

    // Attach only one of two required kinds.
    dur.attach_evidence(&task.id, "test_pass", json!({}), Some(0))
        .await
        .unwrap();

    let task = dur.tasks().get(&task.id).await.unwrap();
    let err = EvidenceGate
        .check(&ctx(&dur, task, TaskStatus::Submitted))
        .await
        .unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("pr_url"),
        "msg should name missing kind: {msg}"
    );
}

#[tokio::test]
async fn evidence_gate_passes_when_all_kinds_present() {
    let (dur, _dir) = fresh().await;
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
                evidence_required: vec!["test_pass".into()],
            },
        )
        .await
        .unwrap();
    dur.attach_evidence(&task.id, "test_pass", json!({}), Some(0))
        .await
        .unwrap();

    let task = dur.tasks().get(&task.id).await.unwrap();
    EvidenceGate
        .check(&ctx(&dur, task, TaskStatus::Submitted))
        .await
        .unwrap();
}

#[tokio::test]
async fn evidence_gate_no_op_for_in_progress_target() {
    // The evidence gate only fires for Submitted/Done. Moving to
    // InProgress must pass even if no evidence is attached.
    let (dur, _dir) = fresh().await;
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
                evidence_required: vec!["test_pass".into()],
            },
        )
        .await
        .unwrap();

    EvidenceGate
        .check(&ctx(&dur, task, TaskStatus::InProgress))
        .await
        .unwrap();
}

#[tokio::test]
async fn wave_sequence_gate_refuses_when_earlier_wave_open() {
    let (dur, _dir) = fresh().await;
    let plan = dur
        .create_plan(NewPlan {
            title: "p".into(),
            description: None,
            project: None,
        })
        .await
        .unwrap();
    let _wave1_task = dur
        .create_task(
            &plan.id,
            NewTask {
                wave: 1,
                sequence: 1,
                title: "w1".into(),
                description: None,
                evidence_required: vec![],
            },
        )
        .await
        .unwrap();
    let wave2_task = dur
        .create_task(
            &plan.id,
            NewTask {
                wave: 2,
                sequence: 1,
                title: "w2".into(),
                description: None,
                evidence_required: vec![],
            },
        )
        .await
        .unwrap();

    let err = WaveSequenceGate
        .check(&ctx(&dur, wave2_task, TaskStatus::InProgress))
        .await
        .unwrap_err();
    matches!(
        err,
        DurabilityError::GateRefused {
            gate: "wave_sequence",
            ..
        }
    );
}

#[tokio::test]
async fn wave_sequence_gate_passes_for_first_wave() {
    let (dur, _dir) = fresh().await;
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
                title: "w1".into(),
                description: None,
                evidence_required: vec![],
            },
        )
        .await
        .unwrap();

    WaveSequenceGate
        .check(&ctx(&dur, task, TaskStatus::InProgress))
        .await
        .unwrap();
}

#[tokio::test]
async fn facade_persists_gate_refusal_for_explanation() {
    let (dur, _dir) = fresh().await;
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
                title: "needs evidence".into(),
                description: None,
                evidence_required: vec!["test".into()],
            },
        )
        .await
        .unwrap();

    dur.transition_task(&task.id, TaskStatus::InProgress, Some("agent"))
        .await
        .unwrap();
    let err = dur
        .transition_task(&task.id, TaskStatus::Submitted, Some("agent"))
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        DurabilityError::GateRefused {
            gate: "evidence",
            ..
        }
    ));

    let refusal = dur
        .audit()
        .latest_refusal(Some(&task.id))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(refusal.transition, "task.refused");
    assert_eq!(refusal.entity_id, task.id);
    assert!(refusal.payload.contains("\"gate\":\"evidence\""));
}
