//! `cvg session ...` — cold-start brief from the daemon.
//!
//! Replaces the static handoff markdown that goes stale. Every value
//! printed here comes from a live daemon query (health, audit, plan
//! tasks) plus an optional `gh pr list` shell-out for the queue.
//!
//! The markdown packet (`docs/agent-resume-packet.md`) is now the
//! TIMELESS half of the cold-start; this command is the time-specific
//! half — read both, in that order, after a session reset.
//!
//! Renderers live in the sibling [`super::session_render`] module to
//! keep both files under the 300-line cap.

use super::session_render::{self, Brief};
use super::{Client, OutputMode};
use anyhow::{anyhow, Context, Result};
use clap::Subcommand;
use convergio_i18n::Bundle;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Command;

/// Session subcommands.
#[derive(Subcommand)]
pub enum SessionCommand {
    /// Print a cold-start brief: daemon health, audit chain, the
    /// active plan, top pending tasks, and open PRs.
    Resume {
        /// Plan id. If omitted, resolves the most recently updated
        /// plan in `--project`.
        plan_id: Option<String>,
        /// Project filter when no plan id is given.
        #[arg(long, default_value = "convergio-local")]
        project: String,
        /// Number of next-priority pending tasks to surface.
        #[arg(long, default_value_t = 5)]
        next_limit: usize,
    },
}

/// Entry point.
pub async fn run(
    client: &Client,
    bundle: &Bundle,
    output: OutputMode,
    cmd: SessionCommand,
) -> Result<()> {
    match cmd {
        SessionCommand::Resume {
            plan_id,
            project,
            next_limit,
        } => resume(client, bundle, output, plan_id, &project, next_limit).await,
    }
}

async fn resume(
    client: &Client,
    bundle: &Bundle,
    output: OutputMode,
    plan_id: Option<String>,
    project: &str,
    next_limit: usize,
) -> Result<()> {
    let health: Value = client.get("/v1/health").await.context("GET /v1/health")?;
    let audit: Value = client
        .get("/v1/audit/verify")
        .await
        .context("GET /v1/audit/verify")?;

    let plan = resolve_plan(client, plan_id.as_deref(), project).await?;
    let tasks: Vec<Task> = client
        .get(&format!("/v1/plans/{}/tasks", plan.id))
        .await
        .context("GET plan tasks")?;
    let counts = TaskCounts::from(tasks.as_slice());
    let next = top_pending(&tasks, next_limit);

    let prs = fetch_open_prs().ok();

    let brief = Brief {
        health: &health,
        audit: &audit,
        plan: &plan,
        counts: &counts,
        next: &next,
        prs: prs.as_deref(),
    };
    session_render::render(bundle, output, &brief)
}

async fn resolve_plan(client: &Client, plan_id: Option<&str>, project: &str) -> Result<Plan> {
    if let Some(id) = plan_id {
        return client
            .get(&format!("/v1/plans/{id}"))
            .await
            .with_context(|| format!("GET /v1/plans/{id}"));
    }
    let plans: Vec<Plan> = client.get("/v1/plans").await.context("GET /v1/plans")?;
    plans
        .into_iter()
        .filter(|p| p.project.as_deref() == Some(project))
        .filter(|p| p.status != "archived")
        .max_by(|a, b| a.updated_at.cmp(&b.updated_at))
        .ok_or_else(|| anyhow!("no active plan found for project={project}"))
}

fn top_pending(tasks: &[Task], limit: usize) -> Vec<Task> {
    let mut pending: Vec<Task> = tasks
        .iter()
        .filter(|t| t.status == "pending")
        .cloned()
        .collect();
    pending.sort_by(|a, b| {
        a.wave
            .cmp(&b.wave)
            .then(a.sequence.cmp(&b.sequence))
            .then(a.created_at.cmp(&b.created_at))
    });
    pending.truncate(limit);
    pending
}

fn fetch_open_prs() -> Result<Vec<PrSummary>> {
    let out = Command::new("gh")
        .args([
            "pr",
            "list",
            "--state",
            "open",
            "--json",
            "number,title,headRefName,isDraft",
        ])
        .output()
        .context("spawn gh")?;
    if !out.status.success() {
        anyhow::bail!(
            "gh pr list failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    serde_json::from_slice(&out.stdout).context("parse gh output")
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct Plan {
    pub(super) id: String,
    pub(super) title: String,
    #[serde(default)]
    pub(super) description: Option<String>,
    #[serde(default)]
    pub(super) project: Option<String>,
    pub(super) status: String,
    pub(super) updated_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(super) struct Task {
    pub(super) id: String,
    pub(super) title: String,
    pub(super) status: String,
    pub(super) wave: i64,
    pub(super) sequence: i64,
    pub(super) created_at: String,
}

#[derive(Debug, Default, Serialize)]
pub(super) struct TaskCounts {
    pub(super) total: usize,
    pub(super) done: usize,
    pub(super) pending: usize,
    pub(super) in_progress: usize,
    pub(super) submitted: usize,
    pub(super) failed: usize,
}

impl From<&[Task]> for TaskCounts {
    fn from(tasks: &[Task]) -> Self {
        let mut c = TaskCounts {
            total: tasks.len(),
            ..Default::default()
        };
        for t in tasks {
            match t.status.as_str() {
                "done" => c.done += 1,
                "pending" => c.pending += 1,
                "in_progress" => c.in_progress += 1,
                "submitted" => c.submitted += 1,
                "failed" => c.failed += 1,
                _ => {}
            }
        }
        c
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct PrSummary {
    pub(super) number: i64,
    pub(super) title: String,
    #[serde(rename = "headRefName")]
    pub(super) head_ref_name: String,
    #[serde(rename = "isDraft", default)]
    pub(super) is_draft: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(status: &str, wave: i64, sequence: i64) -> Task {
        Task {
            id: format!("id-{wave}-{sequence}"),
            title: format!("t{wave}.{sequence}"),
            status: status.into(),
            wave,
            sequence,
            created_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn counts_groups_by_status() {
        let tasks = vec![
            task("done", 1, 1),
            task("pending", 1, 2),
            task("pending", 2, 1),
            task("in_progress", 1, 3),
            task("submitted", 1, 4),
            task("failed", 3, 1),
        ];
        let c = TaskCounts::from(tasks.as_slice());
        assert_eq!(c.total, 6);
        assert_eq!(c.done, 1);
        assert_eq!(c.pending, 2);
        assert_eq!(c.in_progress, 1);
        assert_eq!(c.submitted, 1);
        assert_eq!(c.failed, 1);
    }

    #[test]
    fn top_pending_orders_by_wave_then_sequence() {
        let tasks = vec![
            task("pending", 2, 1),
            task("done", 1, 1),
            task("pending", 1, 5),
            task("pending", 1, 2),
        ];
        let next = top_pending(&tasks, 10);
        let order: Vec<String> = next.iter().map(|t| t.title.clone()).collect();
        assert_eq!(order, vec!["t1.2", "t1.5", "t2.1"]);
    }

    #[test]
    fn top_pending_respects_limit() {
        let tasks: Vec<Task> = (0..10).map(|i| task("pending", 1, i)).collect();
        let next = top_pending(&tasks, 3);
        assert_eq!(next.len(), 3);
    }
}
