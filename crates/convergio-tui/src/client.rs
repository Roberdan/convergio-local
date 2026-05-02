//! HTTP client + GitHub shell-out for the dashboard.
//!
//! Read-only by design. The dashboard never mutates daemon state; if
//! a future iteration adds actions, they belong in a separate module
//! with an explicit ADR (see `AGENTS.md`).
//!
//! ## Endpoints used
//!
//! - `GET /v1/plans` → list of [`Plan`]
//! - `GET /v1/plans/{id}/tasks` → list of [`TaskSummary`] (per plan)
//! - `GET /v1/agents` → list of [`RegistryAgent`]
//! - `GET /v1/audit/verify` → audit chain integrity
//! - `gh pr list` shell-out (skipped when `CONVERGIO_DASH_NO_GH=1`)

use crate::client_gh::fetch_open_prs;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Plan summary, matching the daemon's `/v1/plans` response shape.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Plan {
    /// Plan id.
    pub id: String,
    /// Plan title shown in the Plans pane.
    pub title: String,
    /// Project label (`convergio`, `convergio-local`, ...).
    #[serde(default)]
    pub project: Option<String>,
    /// Plan status (`draft`, `active`, `completed`, ...).
    pub status: String,
    /// Last-updated timestamp (RFC3339).
    pub updated_at: String,
}

/// One task as displayed in the dashboard. The daemon's task shape
/// is richer; we project only what the renderer needs.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TaskSummary {
    /// Task id.
    pub id: String,
    /// Owning plan id.
    pub plan_id: String,
    /// Short title.
    pub title: String,
    /// Status (`pending`, `in_progress`, `submitted`, `done`,
    /// `failed`).
    pub status: String,
    /// Optional agent id that claimed the task.
    #[serde(default)]
    pub agent_id: Option<String>,
}

/// Aggregated counts per status, used by the Plans pane.
#[derive(Debug, Default, Clone, Copy)]
pub struct PlanCounts {
    /// Total tasks observed.
    pub total: usize,
    /// Tasks in `done`.
    pub done: usize,
    /// Tasks in `pending`.
    pub pending: usize,
    /// Tasks in `in_progress`.
    pub in_progress: usize,
    /// Tasks in `submitted` (awaiting Thor).
    pub submitted: usize,
    /// Tasks in `failed`.
    pub failed: usize,
}

impl PlanCounts {
    /// Build from a list of tasks.
    pub fn from_tasks(tasks: &[TaskSummary]) -> Self {
        let mut c = PlanCounts {
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

/// Agent registry row.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RegistryAgent {
    /// Stable agent id.
    pub id: String,
    /// Runner kind (`shell`, `claude`, `copilot`, ...).
    pub kind: String,
    /// `idle`, `working`, `terminated`, ... per registry semantics.
    #[serde(default)]
    pub status: Option<String>,
    /// Last heartbeat (RFC3339), if any.
    #[serde(default)]
    pub last_heartbeat_at: Option<String>,
}

/// Open PR row from `gh pr list`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PrSummary {
    /// PR number.
    pub number: i64,
    /// PR title.
    pub title: String,
    /// Source branch.
    #[serde(rename = "headRefName")]
    pub head_ref_name: String,
    /// Latest CI rollup ("success" / "failure" / "pending" / "").
    #[serde(default)]
    pub ci: String,
}

/// Snapshot of every dataset the dashboard renders.
#[derive(Debug, Default)]
pub struct Snapshot {
    /// Plans known to the daemon.
    pub plans: Vec<Plan>,
    /// Active tasks across plans (`in_progress` + `submitted`).
    pub tasks: Vec<TaskSummary>,
    /// Registered agents.
    pub agents: Vec<RegistryAgent>,
    /// Open pull requests via `gh pr list` (empty when gh disabled).
    pub prs: Vec<PrSummary>,
    /// `Some(true)` if the audit chain verifies, `Some(false)` if not,
    /// `None` if the call could not be made.
    pub audit_ok: Option<bool>,
}

/// Read-only HTTP client. Cloneable.
#[derive(Debug, Clone)]
pub struct Client {
    base: String,
    inner: reqwest::Client,
    enable_gh: bool,
}

impl Client {
    /// Build a client targeting `base` (e.g. `http://127.0.0.1:8420`).
    pub fn new(base: String) -> Self {
        let enable_gh = std::env::var("CONVERGIO_DASH_NO_GH").ok().as_deref() != Some("1");
        Self {
            base,
            inner: reqwest::Client::builder()
                .timeout(Duration::from_secs(3))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            enable_gh,
        }
    }

    /// One-shot fetch of every dataset. Sub-fetches that fail leave
    /// the corresponding field empty rather than aborting the whole
    /// snapshot — partial data is more useful than nothing.
    pub async fn snapshot(&self) -> Result<Snapshot> {
        let plans: Vec<Plan> = self
            .get_json("/v1/plans")
            .await
            .unwrap_or_else(|_| Vec::new());

        let mut tasks: Vec<TaskSummary> = Vec::new();
        for p in &plans {
            if let Ok(mut ts) = self
                .get_json::<Vec<TaskSummary>>(&format!("/v1/plans/{}/tasks", p.id))
                .await
            {
                for t in &mut ts {
                    t.plan_id = p.id.clone();
                }
                tasks.extend(
                    ts.into_iter()
                        .filter(|t| t.status == "in_progress" || t.status == "submitted"),
                );
            }
        }

        let agents: Vec<RegistryAgent> = self
            .get_json("/v1/agent-registry/agents")
            .await
            .unwrap_or_else(|_| Vec::new());

        let audit_ok = self
            .get_json::<serde_json::Value>("/v1/audit/verify")
            .await
            .ok()
            .and_then(|v| v.get("ok").and_then(|b| b.as_bool()));

        let prs = if self.enable_gh {
            fetch_open_prs().unwrap_or_default()
        } else {
            Vec::new()
        };

        Ok(Snapshot {
            plans,
            tasks,
            agents,
            prs,
            audit_ok,
        })
    }

    async fn get_json<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let url = format!("{}{path}", self.base);
        let resp = self
            .inner
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<T>()
            .await?;
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_counts_groups_statuses() {
        let tasks = vec![
            t("done"),
            t("done"),
            t("pending"),
            t("in_progress"),
            t("submitted"),
            t("failed"),
        ];
        let c = PlanCounts::from_tasks(&tasks);
        assert_eq!(c.total, 6);
        assert_eq!(c.done, 2);
        assert_eq!(c.pending, 1);
        assert_eq!(c.in_progress, 1);
        assert_eq!(c.submitted, 1);
        assert_eq!(c.failed, 1);
    }

    fn t(status: &str) -> TaskSummary {
        TaskSummary {
            id: "x".into(),
            plan_id: "p".into(),
            title: "t".into(),
            status: status.into(),
            agent_id: None,
        }
    }
}
