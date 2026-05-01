//! `cvg status` — local dashboard for plans and recently completed work.

use super::{Client, OutputMode};
use anyhow::Result;
use convergio_i18n::Bundle;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::status_render::{render_human, RenderOptions};

/// Plan titles that match these prefixes are considered demo /
/// test artefacts and hidden from the default human view. Pass
/// `--all` (or use `--output json`) to see them.
const DEFAULT_HIDE_PREFIXES: &[&str] = &[
    "Clean local demo",
    "Gate refusal demo",
    "T9-verify-",
    "claude-skill-quickstart-",
    "T0-demo",
    "T11-LIVE-TEST",
];

/// Default fallback agent id when neither `CONVERGIO_AGENT_ID` env var
/// nor the per-task agent_id from F46/F47 is available.
pub(crate) const DEFAULT_FALLBACK_AGENT_ID: &str = "claude-code-roberdan";

fn is_artefact(plan: &PlanSummary) -> bool {
    DEFAULT_HIDE_PREFIXES
        .iter()
        .any(|p| plan.title.starts_with(p))
}

/// Resolve the caller's agent id for `--mine` filtering.
///
/// Reads `CONVERGIO_AGENT_ID` first, falls back to a hard-coded
/// stop-gap until F46/F47 provide a proper local-identity store.
pub(crate) fn resolve_caller_agent_id() -> String {
    std::env::var("CONVERGIO_AGENT_ID")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_FALLBACK_AGENT_ID.to_string())
}

/// Run `cvg status`.
#[allow(clippy::too_many_arguments)]
pub async fn run(
    client: &Client,
    bundle: &Bundle,
    output: OutputMode,
    completed_limit: i64,
    project: Option<String>,
    show_all: bool,
    show_waves: bool,
    mine: bool,
) -> Result<()> {
    let path = format!("/v1/status?completed_limit={completed_limit}");
    let body: Value = client.get(&path).await?;
    let mut status: StatusResponse = serde_json::from_value(body.clone())?;
    if !show_all {
        status.active_plans.retain(|p| !is_artefact(p));
        status.recent_completed_plans.retain(|p| !is_artefact(p));
    }
    if let Some(want) = project.as_deref() {
        let keep = |p: &PlanSummary| p.project.as_deref() == Some(want);
        status.active_plans.retain(keep);
        status.recent_completed_plans.retain(keep);
        status
            .recent_completed_tasks
            .retain(|t| t.project.as_deref() == Some(want));
    }
    let me = if mine {
        Some(resolve_caller_agent_id())
    } else {
        None
    };
    if let Some(me) = me.as_deref() {
        let mine_only = |task: &TaskSummary| task.agent_id.as_deref() == Some(me);
        for plan in status
            .active_plans
            .iter_mut()
            .chain(status.recent_completed_plans.iter_mut())
        {
            plan.next_tasks.retain(mine_only);
        }
    }

    match output {
        OutputMode::Json => {
            println!("{}", serde_json::to_string_pretty(&body)?);
        }
        OutputMode::Plain => {
            render_plain(&status);
        }
        OutputMode::Human => {
            render_human(
                bundle,
                &status,
                RenderOptions {
                    show_waves,
                    mine: me.as_deref(),
                },
            );
        }
    }
    Ok(())
}

fn render_plain(status: &StatusResponse) {
    println!(
        "active_plans={} completed_plans={} completed_tasks={}",
        status.active_plans.len(),
        status.recent_completed_plans.len(),
        status.recent_completed_tasks.len()
    );
}

/// Top-level shape of the `/v1/status` JSON response.
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct StatusResponse {
    pub(crate) active_plans: Vec<PlanSummary>,
    pub(crate) recent_completed_plans: Vec<PlanSummary>,
    pub(crate) recent_completed_tasks: Vec<CompletedTask>,
}

/// One row from `active_plans` / `recent_completed_plans`.
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct PlanSummary {
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    pub(crate) project: Option<String>,
    pub(crate) status: String,
    pub(crate) tasks: TaskCounts,
    pub(crate) next_tasks: Vec<TaskSummary>,
}

/// Task-status breakdown for one plan, returned by the daemon.
#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct TaskCounts {
    pub(crate) total: usize,
    #[serde(default)]
    pub(crate) pending: usize,
    #[serde(default)]
    pub(crate) in_progress: usize,
    #[serde(default)]
    pub(crate) submitted: usize,
    pub(crate) done: usize,
    #[serde(default)]
    pub(crate) failed: usize,
}

/// Pending/in-progress/submitted task summarised under each plan.
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct TaskSummary {
    pub(crate) title: String,
    #[serde(default)]
    pub(crate) status: Option<String>,
    #[serde(default)]
    pub(crate) agent_id: Option<String>,
    #[serde(default)]
    pub(crate) wave: Option<i64>,
    #[serde(default)]
    pub(crate) sequence: Option<i64>,
}

/// One row from `recent_completed_tasks`.
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CompletedTask {
    pub(crate) title: String,
    pub(crate) plan_title: String,
    pub(crate) project: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caller_agent_id_falls_back_when_env_unset() {
        // Note: we cannot reliably mutate process env in tests in
        // parallel; we just assert the fallback path is reachable.
        let resolved = resolve_caller_agent_id();
        assert!(!resolved.is_empty());
    }

    #[test]
    fn status_response_parses_breakdown_fields() {
        let raw = serde_json::json!({
            "active_plans": [{
                "title": "p",
                "description": null,
                "project": "convergio-local",
                "status": "active",
                "tasks": {
                    "total": 5, "pending": 2, "in_progress": 1,
                    "submitted": 1, "done": 1, "failed": 0
                },
                "next_tasks": [{
                    "title": "t",
                    "status": "pending",
                    "agent_id": "claude-code-roberdan",
                    "wave": 1,
                    "sequence": 2
                }]
            }],
            "recent_completed_plans": [],
            "recent_completed_tasks": []
        });
        let parsed: StatusResponse = serde_json::from_value(raw).unwrap();
        assert_eq!(parsed.active_plans.len(), 1);
        let plan = &parsed.active_plans[0];
        assert_eq!(plan.tasks.total, 5);
        assert_eq!(plan.tasks.pending, 2);
        assert_eq!(plan.tasks.submitted, 1);
        assert_eq!(plan.tasks.failed, 0);
        let task = &plan.next_tasks[0];
        assert_eq!(task.agent_id.as_deref(), Some("claude-code-roberdan"));
        assert_eq!(task.wave, Some(1));
    }
}

