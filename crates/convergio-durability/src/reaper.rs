//! Reaper loop.
//!
//! The reaper periodically scans `tasks` for rows in `in_progress`
//! whose `last_heartbeat_at` is older than [`ReaperConfig::timeout`],
//! moves them back to `pending`, clears `agent_id`, and writes one
//! `task.reaped` audit row per release.
//!
//! There is **exactly one** of these per daemon. If you find yourself
//! adding a second background loop in Layer 1, stop and consider
//! whether it belongs in a Layer 4 crate instead — see
//! [ARCHITECTURE.md](../../../ARCHITECTURE.md) § "Background loops".

use crate::audit::EntityKind;
use crate::error::Result;
use crate::facade::Durability;
use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

/// Reaper configuration.
#[derive(Debug, Clone)]
pub struct ReaperConfig {
    /// Heartbeat older than this releases the task.
    pub timeout: Duration,
    /// How often the loop ticks.
    pub tick_interval: Duration,
}

impl Default for ReaperConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::seconds(300),
            tick_interval: Duration::seconds(60),
        }
    }
}

/// Spawned-loop handle. Drop the handle to abort the loop.
pub struct ReaperHandle {
    inner: JoinHandle<()>,
}

impl ReaperHandle {
    /// Abort the loop. Idempotent.
    pub fn abort(self) {
        self.inner.abort();
    }
}

/// Spawn the reaper loop and return its handle.
///
/// The loop is fire-and-forget: errors during a tick are logged at
/// `warn!` and do **not** kill the loop. Persistent issues should
/// surface via metrics, not by silent loop death.
pub fn spawn(durability: Arc<Durability>, config: ReaperConfig) -> ReaperHandle {
    let inner = tokio::spawn(async move {
        info!(?config, "reaper started");
        let interval = tokio_duration(config.tick_interval);
        loop {
            tokio::time::sleep(interval).await;
            match tick(&durability, &config).await {
                Ok(n) if n > 0 => info!(reaped = n, "reaper tick"),
                Ok(_) => debug!("reaper tick: nothing stale"),
                Err(e) => warn!(error = %e, "reaper tick failed"),
            }
        }
    });
    ReaperHandle { inner }
}

/// Run one tick. Returns the number of tasks released.
///
/// Exposed for tests and for callers that want to drive the loop on
/// their own schedule (e.g. a manual `cvg doctor reap`).
pub async fn tick(durability: &Durability, config: &ReaperConfig) -> Result<usize> {
    let cutoff = Utc::now() - config.timeout;
    let stale = find_stale(durability, cutoff).await?;

    let mut released = 0usize;
    for (id, agent_id) in stale {
        release_one(durability, &id, agent_id.as_deref(), &cutoff).await?;
        released += 1;
    }
    Ok(released)
}

async fn find_stale(
    durability: &Durability,
    cutoff: DateTime<Utc>,
) -> Result<Vec<(String, Option<String>)>> {
    let cutoff_str = cutoff.to_rfc3339();
    let rows = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT id, agent_id FROM tasks \
         WHERE status = 'in_progress' \
           AND last_heartbeat_at IS NOT NULL \
           AND last_heartbeat_at < ?",
    )
    .bind(&cutoff_str)
    .fetch_all(durability.pool().inner())
    .await?;
    Ok(rows)
}

async fn release_one(
    durability: &Durability,
    task_id: &str,
    agent_id: Option<&str>,
    cutoff: &DateTime<Utc>,
) -> Result<()> {
    sqlx::query(
        "UPDATE tasks SET status = 'pending', agent_id = NULL, updated_at = ? WHERE id = ?",
    )
    .bind(Utc::now().to_rfc3339())
    .bind(task_id)
    .execute(durability.pool().inner())
    .await?;

    durability
        .audit()
        .append(
            EntityKind::Task,
            task_id,
            "task.reaped",
            &json!({
                "task_id": task_id,
                "previous_agent_id": agent_id,
                "cutoff": cutoff.to_rfc3339(),
            }),
            None,
        )
        .await?;
    Ok(())
}

fn tokio_duration(d: Duration) -> std::time::Duration {
    std::time::Duration::from_millis(d.num_milliseconds().max(1) as u64)
}
