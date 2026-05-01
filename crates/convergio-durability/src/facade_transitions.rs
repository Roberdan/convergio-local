//! Task transition methods on [`Durability`].
//!
//! Split out of `facade.rs` to keep both files under the 300-line cap.
//! These methods all share the same invariants:
//!
//! 1. Every state-changing call writes exactly one audit row (or zero
//!    on a refusal that is itself recorded as `task.refused`).
//! 2. `done` is never reachable through [`Durability::transition_task`];
//!    it is set only by [`Durability::complete_validated_tasks`], which
//!    Thor calls on a Pass verdict (CONSTITUTION §6, ADR-0011).

use crate::audit::{append_tx, EntityKind};
use crate::error::{DurabilityError, Result};
use crate::facade::Durability;
use crate::gates::{self, GateContext};
use crate::model::{Task, TaskStatus};
use chrono::Utc;
use serde_json::json;

impl Durability {
    /// Move a task to a new status, running the gate pipeline first.
    /// On success, writes one audit row.
    ///
    /// `target = TaskStatus::Done` is **never** accepted here. `done`
    /// is a verdict produced by [`Self::complete_validated_tasks`]
    /// (called by Thor on a Pass verdict). See CONSTITUTION §6 and
    /// ADR-0011. A `Done` target produces an audit row of kind
    /// `task.refused` and returns [`DurabilityError::DoneNotByThor`].
    pub async fn transition_task(
        &self,
        task_id: &str,
        target: TaskStatus,
        agent_id: Option<&str>,
    ) -> Result<Task> {
        let task = self.tasks().get(task_id).await?;
        if matches!(target, TaskStatus::Done) {
            self.record_done_refusal(&task, agent_id).await?;
            return Err(DurabilityError::DoneNotByThor);
        }
        let ctx = GateContext {
            pool: self.pool().clone(),
            task: task.clone(),
            target_status: target,
            agent_id: agent_id.map(str::to_string),
        };
        if let Err(e) = gates::run(self.pipeline(), &ctx).await {
            if let DurabilityError::GateRefused { gate, reason } = &e {
                self.record_gate_refusal(&task, target, agent_id, gate, reason)
                    .await?;
            }
            return Err(e);
        }

        let mut tx = self.pool().inner().begin().await?;
        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE tasks SET status = ?, agent_id = ?, updated_at = ? WHERE id = ?")
            .bind(target.as_str())
            .bind(agent_id)
            .bind(&now)
            .bind(task_id)
            .execute(&mut *tx)
            .await?;
        sync_agent_current_task(&mut tx, &task, target, agent_id, &now).await?;
        append_tx(
            &mut tx,
            EntityKind::Task,
            task_id,
            &format!("task.{}", target.as_str()),
            &json!({
                "task_id": task_id,
                "from": task.status.as_str(),
                "to": target.as_str(),
                "agent_id": agent_id,
            }),
            agent_id,
        )
        .await?;
        tx.commit().await?;
        self.tasks().get(task_id).await
    }
}

/// Mirror the task transition into the `agents` row of the agent that
/// is gaining or losing the task (closes F46).
///
/// - On `target == InProgress` with an `agent_id`, marks that agent as
///   `working` and points its `current_task_id` at the task.
/// - On a transition *out of* `InProgress`, clears the previous owner's
///   `current_task_id` and flips them back to `idle`, but only if their
///   `current_task_id` still matches this task — guards against the
///   case where the agent has already moved on to another claim.
///
/// Silent no-op when the agent is not registered in the `agents`
/// table (UPDATE just affects zero rows). The whole edit shares the
/// caller's transaction so the agents row, the tasks row, and the
/// audit row commit together.
async fn sync_agent_current_task(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    task: &crate::model::Task,
    target: crate::model::TaskStatus,
    agent_id: Option<&str>,
    now: &str,
) -> Result<()> {
    use crate::model::TaskStatus;
    if matches!(target, TaskStatus::InProgress) {
        if let Some(aid) = agent_id {
            sqlx::query(
                "UPDATE agents \
                 SET current_task_id = ?, status = 'working', updated_at = ? \
                 WHERE id = ?",
            )
            .bind(&task.id)
            .bind(now)
            .bind(aid)
            .execute(&mut **tx)
            .await?;
        }
        return Ok(());
    }
    if matches!(task.status, TaskStatus::InProgress) {
        if let Some(prev) = task.agent_id.as_deref() {
            sqlx::query(
                "UPDATE agents \
                 SET current_task_id = NULL, status = 'idle', updated_at = ? \
                 WHERE id = ? AND current_task_id = ?",
            )
            .bind(now)
            .bind(prev)
            .bind(&task.id)
            .execute(&mut **tx)
            .await?;
        }
    }
    Ok(())
}

impl Durability {
    /// Promote a set of `submitted` tasks to `done` atomically.
    ///
    /// Reserved for the validator (Thor) — invoked from
    /// `Thor::validate` only after every task in the plan has passed
    /// evidence checks. Skips the gate pipeline because gates already
    /// ran on the `submitted` transition.
    ///
    /// Each promoted task gets one audit row of kind
    /// `task.completed_by_thor`. The whole batch is one transaction so
    /// either every task in `task_ids` flips or none do.
    ///
    /// Returns the list of completed tasks in input order. Errors with
    /// [`DurabilityError::NotSubmitted`] if any task is not currently
    /// in `submitted`.
    pub async fn complete_validated_tasks(&self, task_ids: &[String]) -> Result<Vec<Task>> {
        if task_ids.is_empty() {
            return Ok(Vec::new());
        }
        let now = Utc::now();
        let mut tx = self.pool().inner().begin().await?;
        for id in task_ids {
            let row: (String,) = sqlx::query_as("SELECT status FROM tasks WHERE id = ?")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| DurabilityError::NotFound {
                    entity: "task",
                    id: id.clone(),
                })?;
            let status = TaskStatus::parse(&row.0).ok_or_else(|| DurabilityError::NotFound {
                entity: "task_status",
                id: row.0.clone(),
            })?;
            if !matches!(status, TaskStatus::Submitted) {
                return Err(DurabilityError::NotSubmitted {
                    id: id.clone(),
                    actual: status.as_str(),
                });
            }
            sqlx::query("UPDATE tasks SET status = ?, updated_at = ? WHERE id = ?")
                .bind(TaskStatus::Done.as_str())
                .bind(now.to_rfc3339())
                .bind(id)
                .execute(&mut *tx)
                .await?;
            append_tx(
                &mut tx,
                EntityKind::Task,
                id,
                "task.completed_by_thor",
                &json!({
                    "task_id": id,
                    "from": "submitted",
                    "to": "done",
                }),
                None,
            )
            .await?;
        }
        tx.commit().await?;
        let mut completed = Vec::with_capacity(task_ids.len());
        for id in task_ids {
            completed.push(self.tasks().get(id).await?);
        }
        Ok(completed)
    }

    pub(crate) async fn record_done_refusal(
        &self,
        task: &Task,
        agent_id: Option<&str>,
    ) -> Result<()> {
        self.audit()
            .append(
                EntityKind::Task,
                &task.id,
                "task.refused",
                &json!({
                    "task_id": task.id,
                    "from": task.status.as_str(),
                    "to": "done",
                    "reason": "done is set only by validation (cvg validate)",
                    "agent_id": agent_id,
                }),
                agent_id,
            )
            .await?;
        Ok(())
    }

    pub(crate) async fn record_gate_refusal(
        &self,
        task: &Task,
        target: TaskStatus,
        agent_id: Option<&str>,
        gate: &str,
        reason: &str,
    ) -> Result<()> {
        self.audit()
            .append(
                EntityKind::Task,
                &task.id,
                "task.refused",
                &json!({
                    "task_id": task.id,
                    "from": task.status.as_str(),
                    "to": target.as_str(),
                    "gate": gate,
                    "reason": reason,
                    "agent_id": agent_id,
                }),
                agent_id,
            )
            .await?;
        Ok(())
    }
}
