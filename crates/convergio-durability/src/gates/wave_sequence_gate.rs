//! `WaveSequenceGate` — refuses `in_progress` claims for tasks in a
//! wave whose predecessor wave is not yet fully complete.

use super::{Gate, GateContext};
use crate::error::{DurabilityError, Result};
use crate::model::TaskStatus;

/// Tasks in wave N+1 cannot start until every task in wave N is `done`.
pub struct WaveSequenceGate;

#[async_trait::async_trait]
impl Gate for WaveSequenceGate {
    fn name(&self) -> &'static str {
        "wave_sequence"
    }

    async fn check(&self, ctx: &GateContext) -> Result<()> {
        if !matches!(ctx.target_status, TaskStatus::InProgress) {
            return Ok(());
        }
        if ctx.task.wave <= 1 {
            return Ok(());
        }

        let row = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM tasks \
             WHERE plan_id = ? AND wave < ? AND status != 'done'",
        )
        .bind(&ctx.task.plan_id)
        .bind(ctx.task.wave)
        .fetch_one(ctx.pool.inner())
        .await?;

        if row.0 > 0 {
            Err(DurabilityError::GateRefused {
                gate: "wave_sequence",
                reason: format!("{} task(s) in earlier waves still open", row.0),
            })
        } else {
            Ok(())
        }
    }
}
