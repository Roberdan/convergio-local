//! `Executor::tick` — one-shot dispatch round.

use crate::error::Result;
use chrono::Duration;
use convergio_durability::{Durability, TaskStatus};
use convergio_lifecycle::{SpawnSpec, Supervisor};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

/// Template the executor uses to spawn agents.
///
/// In the MVP every task spawns the same template — `command` with
/// `args` plus the task id appended. A future milestone will allow
/// per-task templates (e.g. coming from a Plan-level `agent_kind`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnTemplate {
    /// argv0.
    pub command: String,
    /// argv[1..n] — the task id is appended after these.
    pub args: Vec<String>,
    /// Logical kind tag (passed through to `agent_processes.kind`).
    pub kind: String,
}

impl Default for SpawnTemplate {
    fn default() -> Self {
        Self {
            command: "/bin/echo".into(),
            args: vec!["task".into()],
            kind: "shell".into(),
        }
    }
}

/// Executor handle.
#[derive(Clone)]
pub struct Executor {
    durability: Durability,
    supervisor: Supervisor,
    template: SpawnTemplate,
}

impl Executor {
    /// Build with the given facades and spawn template.
    pub fn new(durability: Durability, supervisor: Supervisor, template: SpawnTemplate) -> Self {
        Self {
            durability,
            supervisor,
            template,
        }
    }

    /// Run one dispatch round. Returns the number of tasks moved to
    /// `in_progress`.
    pub async fn tick(&self) -> Result<usize> {
        let pending = self.find_dispatchable().await?;
        let mut dispatched = 0usize;
        for (task_id, plan_id) in pending {
            self.dispatch_one(&task_id, &plan_id).await?;
            dispatched += 1;
        }
        Ok(dispatched)
    }

    async fn find_dispatchable(&self) -> Result<Vec<(String, String)>> {
        // Pending tasks whose wave is "ready" — no earlier-wave task
        // is still open in the same plan.
        let rows = sqlx::query_as::<_, (String, String, i64)>(
            "SELECT t.id, t.plan_id, t.wave \
             FROM tasks t \
             WHERE t.status = 'pending' \
               AND NOT EXISTS ( \
                   SELECT 1 FROM tasks t2 \
                   WHERE t2.plan_id = t.plan_id \
                     AND t2.wave < t.wave \
                      AND t2.status NOT IN ('done', 'failed') \
               ) \
             ORDER BY t.wave ASC, t.sequence ASC",
        )
        .fetch_all(self.durability.pool().inner())
        .await
        .map_err(convergio_durability::DurabilityError::from)?;
        Ok(rows.into_iter().map(|r| (r.0, r.1)).collect())
    }

    async fn dispatch_one(&self, task_id: &str, plan_id: &str) -> Result<()> {
        let mut args = self.template.args.clone();
        args.push(task_id.to_string());

        let proc = self
            .supervisor
            .spawn(SpawnSpec {
                kind: self.template.kind.clone(),
                command: self.template.command.clone(),
                args,
                env: vec![],
                plan_id: Some(plan_id.to_string()),
                task_id: Some(task_id.to_string()),
            })
            .await?;

        self.durability
            .transition_task(task_id, TaskStatus::InProgress, Some(&proc.id))
            .await?;
        Ok(())
    }
}

/// Spawned-loop handle. Drop the handle to abort.
pub struct ExecutorHandle {
    inner: JoinHandle<()>,
}

impl ExecutorHandle {
    /// Abort the loop. Idempotent.
    pub fn abort(&self) {
        self.inner.abort();
    }
}

/// Spawn the executor loop. Errors during a tick are logged at
/// `warn!` and do not kill the loop.
pub fn spawn_loop(executor: Arc<Executor>, tick_interval: Duration) -> ExecutorHandle {
    let inner = tokio::spawn(async move {
        info!(
            tick_secs = tick_interval.num_seconds(),
            "executor loop started"
        );
        let interval = tokio_duration(tick_interval);
        loop {
            tokio::time::sleep(interval).await;
            match executor.tick().await {
                Ok(n) if n > 0 => info!(dispatched = n, "executor tick"),
                Ok(_) => debug!("executor tick: nothing pending"),
                Err(e) => warn!(error = %e, "executor tick failed"),
            }
        }
    });
    ExecutorHandle { inner }
}

fn tokio_duration(d: Duration) -> std::time::Duration {
    std::time::Duration::from_millis(d.num_milliseconds().max(1) as u64)
}
