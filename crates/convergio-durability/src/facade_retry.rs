//! `Durability::retry_task` — administrative recovery for failed tasks.
//!
//! Split out of `facade_transitions.rs` to respect the 300-line cap.
//! Closes friction-log F38: `Failed` was advertised as terminal in the
//! original model doc, but in practice every fix that lands after a
//! failure needs a way to reopen the work. The retry path skips the
//! gate pipeline (gates re-run on the next
//! `pending → in_progress → submitted` walk) and writes exactly one
//! audit row of kind `task.retried`.

use crate::audit::{append_tx, EntityKind};
use crate::error::{DurabilityError, Result};
use crate::facade::Durability;
use crate::model::{Task, TaskStatus};
use chrono::Utc;
use serde_json::json;

impl Durability {
    /// Move a task from `failed` back to `pending`, clearing its
    /// previous owner so a new agent can claim it.
    ///
    /// Errors with [`DurabilityError::NotFailed`] if the task is in
    /// any other status.
    pub async fn retry_task(&self, task_id: &str, agent_id: Option<&str>) -> Result<Task> {
        let task = self.tasks().get(task_id).await?;
        if !matches!(task.status, TaskStatus::Failed) {
            return Err(DurabilityError::NotFailed {
                id: task_id.to_string(),
                actual: task.status.as_str(),
            });
        }
        let mut tx = self.pool().inner().begin().await?;
        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE tasks SET status = ?, agent_id = NULL, updated_at = ? WHERE id = ?")
            .bind(TaskStatus::Pending.as_str())
            .bind(&now)
            .bind(task_id)
            .execute(&mut *tx)
            .await?;
        append_tx(
            &mut tx,
            EntityKind::Task,
            task_id,
            "task.retried",
            &json!({
                "task_id": task_id,
                "from": "failed",
                "to": "pending",
                "agent_id": agent_id,
            }),
            agent_id,
        )
        .await?;
        tx.commit().await?;
        self.tasks().get(task_id).await
    }
}
