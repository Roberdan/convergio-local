//! Plan lifecycle transitions on [`Durability`].
//!
//! Allowed graph: `draft → active`, `draft → cancelled`,
//! `active → completed`, `active → cancelled`. Anything else is
//! refused with [`DurabilityError::IllegalPlanTransition`]. Idempotent
//! on `from == to`. Each accepted transition writes one audit row of
//! kind `plan.<target>`.
//!
//! Behaviour is exercised end-to-end in
//! `convergio-server/tests/e2e_plan_transition.rs`.

use crate::audit::{append_tx, EntityKind};
use crate::error::{DurabilityError, Result};
use crate::facade::Durability;
use crate::model::{Plan, PlanStatus};
use chrono::Utc;
use serde_json::json;

impl Durability {
    /// Move a plan to `target` if the transition is allowed.
    pub async fn transition_plan(&self, plan_id: &str, target: PlanStatus) -> Result<Plan> {
        let plan = self.plans().get(plan_id).await?;
        let legal = plan.status == target
            || matches!(
                (plan.status, target),
                (PlanStatus::Draft, PlanStatus::Active)
                    | (PlanStatus::Draft, PlanStatus::Cancelled)
                    | (PlanStatus::Active, PlanStatus::Completed)
                    | (PlanStatus::Active, PlanStatus::Cancelled)
            );
        if !legal {
            return Err(DurabilityError::IllegalPlanTransition {
                from: plan.status.as_str(),
                to: target.as_str(),
            });
        }
        if plan.status == target {
            return Ok(plan);
        }
        let mut tx = self.pool().inner().begin().await?;
        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE plans SET status = ?, updated_at = ? WHERE id = ?")
            .bind(target.as_str())
            .bind(&now)
            .bind(plan_id)
            .execute(&mut *tx)
            .await?;
        append_tx(
            &mut tx,
            EntityKind::Plan,
            plan_id,
            &format!("plan.{}", target.as_str()),
            &json!({
                "plan_id": plan_id,
                "from": plan.status.as_str(),
                "to": target.as_str(),
            }),
            None,
        )
        .await?;
        tx.commit().await?;
        self.plans().get(plan_id).await
    }
}
