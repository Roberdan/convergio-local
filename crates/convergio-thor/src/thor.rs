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

    /// Read every task of `plan_id`, check that each is `done` with
    /// the required evidence kinds present, and return a verdict.
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
        for task in tasks {
            if task.status != TaskStatus::Done {
                reasons.push(format!(
                    "task {} ({}) is {} — expected done",
                    task.id,
                    task.title,
                    task.status.as_str()
                ));
                continue;
            }
            let kinds = self.durability.evidence().kinds_for(&task.id).await?;
            for required in &task.evidence_required {
                if !kinds.iter().any(|k| k == required) {
                    reasons.push(format!(
                        "task {} ({}) missing evidence kind '{}'",
                        task.id, task.title, required
                    ));
                }
            }
        }

        if reasons.is_empty() {
            Ok(Verdict::Pass)
        } else {
            Ok(Verdict::Fail { reasons })
        }
    }
}
