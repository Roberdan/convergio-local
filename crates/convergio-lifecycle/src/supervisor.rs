//! Supervisor — spawns processes and tracks them in the DB.

use crate::error::{LifecycleError, Result};
use crate::model::{AgentProcess, ProcessStatus, SpawnSpec};
use chrono::{DateTime, Utc};
use convergio_db::Pool;
use std::process::Stdio;
use tokio::process::Command;
use uuid::Uuid;

/// Read/write access to `agent_processes` + spawn capability.
#[derive(Clone)]
pub struct Supervisor {
    pool: Pool,
}

impl Supervisor {
    /// Wrap a pool. The caller is responsible for having run
    /// [`crate::init`] at least once.
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Spawn a process and persist its row.
    ///
    /// The function returns as soon as the OS PID is captured. The
    /// process keeps running detached. A future Layer 3 milestone will
    /// add a watcher task that updates `status` on exit.
    pub async fn spawn(&self, spec: SpawnSpec) -> Result<AgentProcess> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let now_str = now.to_rfc3339();

        sqlx::query(
            "INSERT INTO agent_processes (id, kind, command, plan_id, task_id, pid, \
             status, exit_code, last_heartbeat_at, started_at, ended_at) \
             VALUES (?, ?, ?, ?, ?, NULL, 'starting', NULL, NULL, ?, NULL)",
        )
        .bind(&id)
        .bind(&spec.kind)
        .bind(&spec.command)
        .bind(&spec.plan_id)
        .bind(&spec.task_id)
        .bind(&now_str)
        .execute(self.pool.inner())
        .await?;

        let mut cmd = Command::new(&spec.command);
        cmd.args(&spec.args);
        for (k, v) in &spec.env {
            cmd.env(k, v);
        }
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());

        let child = cmd
            .spawn()
            .map_err(|e| LifecycleError::SpawnFailed(format!("{}: {e}", spec.command)))?;
        let pid = child.id().map(|p| p as i64);

        sqlx::query("UPDATE agent_processes SET pid = ?, status = 'running' WHERE id = ?")
            .bind(pid)
            .bind(&id)
            .execute(self.pool.inner())
            .await?;

        // Drop the child so tokio doesn't kill it on drop. The OS owns
        // the process from here; reaping happens in a follow-up loop.
        drop(child);

        Ok(AgentProcess {
            id,
            kind: spec.kind,
            command: spec.command,
            plan_id: spec.plan_id,
            task_id: spec.task_id,
            pid,
            status: ProcessStatus::Running,
            exit_code: None,
            last_heartbeat_at: None,
            started_at: now,
            ended_at: None,
        })
    }

    /// Touch `last_heartbeat_at` for the given process.
    pub async fn heartbeat(&self, id: &str) -> Result<()> {
        let n = sqlx::query("UPDATE agent_processes SET last_heartbeat_at = ? WHERE id = ?")
            .bind(Utc::now().to_rfc3339())
            .bind(id)
            .execute(self.pool.inner())
            .await?
            .rows_affected();
        if n == 0 {
            return Err(LifecycleError::NotFound(id.to_string()));
        }
        Ok(())
    }

    /// Mark a process terminated with the given exit code and status.
    /// Used by tests and (future) the OS-watcher.
    pub async fn mark_exited(&self, id: &str, exit_code: Option<i64>, ok: bool) -> Result<()> {
        let status = if ok {
            ProcessStatus::Exited
        } else {
            ProcessStatus::Failed
        };
        let n = sqlx::query(
            "UPDATE agent_processes SET status = ?, exit_code = ?, ended_at = ? WHERE id = ?",
        )
        .bind(status.as_str())
        .bind(exit_code)
        .bind(Utc::now().to_rfc3339())
        .bind(id)
        .execute(self.pool.inner())
        .await?
        .rows_affected();
        if n == 0 {
            return Err(LifecycleError::NotFound(id.to_string()));
        }
        Ok(())
    }

    /// Fetch one row by id.
    pub async fn get(&self, id: &str) -> Result<AgentProcess> {
        let row = sqlx::query_as::<_, ProcessRow>(
            "SELECT id, kind, command, plan_id, task_id, pid, status, exit_code, \
             last_heartbeat_at, started_at, ended_at FROM agent_processes \
             WHERE id = ? LIMIT 1",
        )
        .bind(id)
        .fetch_optional(self.pool.inner())
        .await?;
        row.ok_or_else(|| LifecycleError::NotFound(id.to_string()))?
            .try_into()
    }
}

#[derive(sqlx::FromRow)]
struct ProcessRow {
    id: String,
    kind: String,
    command: String,
    plan_id: Option<String>,
    task_id: Option<String>,
    pid: Option<i64>,
    status: String,
    exit_code: Option<i64>,
    last_heartbeat_at: Option<String>,
    started_at: String,
    ended_at: Option<String>,
}

impl TryFrom<ProcessRow> for AgentProcess {
    type Error = LifecycleError;
    fn try_from(r: ProcessRow) -> Result<Self> {
        Ok(AgentProcess {
            id: r.id,
            kind: r.kind,
            command: r.command,
            plan_id: r.plan_id,
            task_id: r.task_id,
            pid: r.pid,
            status: ProcessStatus::parse(&r.status).unwrap_or(ProcessStatus::Failed),
            exit_code: r.exit_code,
            last_heartbeat_at: r.last_heartbeat_at.as_deref().and_then(parse_ts_opt),
            started_at: parse_ts(&r.started_at)?,
            ended_at: r.ended_at.as_deref().and_then(parse_ts_opt),
        })
    }
}

fn parse_ts(s: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|_| LifecycleError::NotFound(format!("bad timestamp: {s}")))
}

fn parse_ts_opt(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|d| d.with_timezone(&Utc))
}
