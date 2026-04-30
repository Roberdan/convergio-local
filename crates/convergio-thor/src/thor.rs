//! `Thor::validate` — produce a verdict for a plan.

use crate::error::Result;
use convergio_durability::{Durability, TaskStatus};
use serde::{Deserialize, Serialize};

/// Validator verdict.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "verdict")]
pub enum Verdict {
    /// Plan passes — safe to close.
    Pass,
    /// Plan fails — list of reasons (one per failing task / missing
    /// evidence kind).
    Fail {
        /// Why the plan was rejected.
        reasons: Vec<String>,
    },
}

/// Thor validator handle.
#[derive(Clone)]
pub struct Thor {
    durability: Durability,
}

impl Thor {
    /// Wrap a [`Durability`] facade.
    pub fn new(durability: Durability) -> Self {
        Self { durability }
    }

    /// Validate every task of `plan_id`. A task is valid when:
    ///
    /// 1. its status is `submitted` or `done` (anything else fails);
    /// 2. every kind listed in `evidence_required` has at least one
    ///    matching evidence row.
    ///
    /// On a passing verdict, every task currently in `submitted` is
    /// promoted to `done` atomically through
    /// [`Durability::complete_validated_tasks`]. This is the **only**
    /// path that sets `done` (CONSTITUTION §6, ADR-0011) — agents may
    /// never self-promote via `transition_task`.
    ///
    /// The verdict is idempotent: validating a plan whose tasks are
    /// already all `done` simply returns `Pass` with zero promotions.
    pub async fn validate(&self, plan_id: &str) -> Result<Verdict> {
        // Confirm the plan exists — yields NotFound otherwise.
        self.durability.plans().get(plan_id).await?;

        let tasks = self.durability.tasks().list_by_plan(plan_id).await?;
        if tasks.is_empty() {
            return Ok(Verdict::Fail {
                reasons: vec!["plan has no tasks".into()],
            });
        }

        let mut reasons = Vec::new();
        let mut to_promote: Vec<String> = Vec::new();
        for task in tasks {
            match task.status {
                TaskStatus::Submitted | TaskStatus::Done => {}
                TaskStatus::Failed => {
                    reasons.push(format!(
                        "task {} ({}) is failed — plan cannot validate",
                        task.id, task.title
                    ));
                    continue;
                }
                _ => {
                    reasons.push(format!(
                        "task {} ({}) is {} — expected submitted or done",
                        task.id,
                        task.title,
                        task.status.as_str()
                    ));
                    continue;
                }
            }
            let kinds = self.durability.evidence().kinds_for(&task.id).await?;
            let mut task_ok = true;
            for required in &task.evidence_required {
                if !kinds.iter().any(|k| k == required) {
                    reasons.push(format!(
                        "task {} ({}) missing evidence kind '{}'",
                        task.id, task.title, required
                    ));
                    task_ok = false;
                }
            }
            if task_ok && matches!(task.status, TaskStatus::Submitted) {
                to_promote.push(task.id.clone());
            }
        }

        if !reasons.is_empty() {
            return Ok(Verdict::Fail { reasons });
        }

        // Pass: promote every still-submitted task to done atomically.
        // Empty list is a no-op (idempotent re-validate).
        self.durability
            .complete_validated_tasks(&to_promote)
            .await?;
        Ok(Verdict::Pass)
    }
}
