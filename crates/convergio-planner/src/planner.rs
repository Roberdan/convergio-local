//! `Planner::solve` — turn a mission into a plan + tasks.

use crate::error::{PlannerError, Result};
use convergio_durability::{Durability, NewPlan, NewTask};

/// Planner facade.
#[derive(Clone)]
pub struct Planner {
    durability: Durability,
}

impl Planner {
    /// Wrap a [`Durability`] facade.
    pub fn new(durability: Durability) -> Self {
        Self { durability }
    }

    /// Take a mission string, write a plan with one task per
    /// non-blank line, return the plan id.
    pub async fn solve(&self, mission: &str) -> Result<String> {
        let lines: Vec<&str> = mission
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();
        if lines.is_empty() {
            return Err(PlannerError::EmptyMission);
        }

        let title = lines[0].to_string();
        let description = if lines.len() > 1 {
            Some(lines[1..].join("\n"))
        } else {
            None
        };

        let plan = self
            .durability
            .create_plan(NewPlan {
                org_id: "default".into(),
                title,
                description,
            })
            .await?;

        for (i, line) in lines.iter().enumerate() {
            self.durability
                .create_task(
                    &plan.id,
                    NewTask {
                        wave: 1,
                        sequence: (i + 1) as i64,
                        title: (*line).to_string(),
                        description: None,
                        evidence_required: vec![],
                    },
                )
                .await?;
        }

        Ok(plan.id)
    }
}
