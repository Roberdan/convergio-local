//! `cvg status` — local dashboard for plans and recently completed work.

use super::{Client, OutputMode};
use anyhow::Result;
use convergio_i18n::Bundle;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

fn is_artefact(plan: &PlanSummary) -> bool {
    DEFAULT_HIDE_PREFIXES
        .iter()
        .any(|p| plan.title.starts_with(p))
}

/// Run `cvg status`.
pub async fn run(
    client: &Client,
    bundle: &Bundle,
    output: OutputMode,
    completed_limit: i64,
    project: Option<String>,
    show_all: bool,
    show_agents: bool,
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

    let agents = if show_agents {
        Some(fetch_agents(client).await?)
    } else {
        None
    };

    match output {
        OutputMode::Json => {
            let mut combined = body;
            if let Some(list) = agents.as_ref() {
                combined["agents"] = serde_json::to_value(list)?;
            }
            println!("{}", serde_json::to_string_pretty(&combined)?);
        }
        OutputMode::Plain => {
            render_plain(&status);
            if let Some(list) = agents.as_ref() {
                render_agents_plain(list);
            }
        }
        OutputMode::Human => {
            render_human(bundle, &status);
            if let Some(list) = agents.as_ref() {
                render_agents_human(bundle, list);
            }
        }
    }
    Ok(())
}

async fn fetch_agents(client: &Client) -> Result<Vec<AgentSummary>> {
    let body: Value = client.get("/v1/agent-registry/agents").await?;
    let agents: Vec<AgentSummary> = serde_json::from_value(body)?;
    Ok(agents)
}

fn render_agents_plain(agents: &[AgentSummary]) {
    println!("agents={}", agents.len());
    for a in agents {
        println!(
            "agent id={} kind={} status={} host={} task={}",
            a.id,
            a.kind,
            a.status,
            a.host.as_deref().unwrap_or("-"),
            a.current_task_id.as_deref().unwrap_or("-")
        );
    }
}

fn render_agents_human(bundle: &Bundle, agents: &[AgentSummary]) {
    if agents.is_empty() {
        println!("{}", bundle.t("status-agents-empty", &[]));
        return;
    }
    println!("{}", bundle.t("status-agents-header", &[]));
    for a in agents {
        let last = a.last_heartbeat_at.as_deref().unwrap_or("-");
        println!(
            "{}",
            bundle.t(
                "status-agent-line",
                &[
                    ("id", &a.id),
                    ("kind", &a.kind),
                    ("host", a.host.as_deref().unwrap_or("-")),
                    ("status", &a.status),
                    ("task", a.current_task_id.as_deref().unwrap_or("-")),
                    ("last_heartbeat", last),
                ],
            )
        );
    }
}

fn render_plain(status: &StatusResponse) {
    println!(
        "active_plans={} completed_plans={} completed_tasks={}",
        status.active_plans.len(),
        status.recent_completed_plans.len(),
        status.recent_completed_tasks.len()
    );
}

fn render_human(bundle: &Bundle, status: &StatusResponse) {
    println!("{}", bundle.t("status-header", &[]));
    if status.active_plans.is_empty() {
        println!("{}", bundle.t("status-active-empty", &[]));
    } else {
        println!("{}", bundle.t("status-active-header", &[]));
        for plan in &status.active_plans {
            print_plan(bundle, plan);
        }
    }

    if status.recent_completed_plans.is_empty() {
        println!("{}", bundle.t("status-completed-empty", &[]));
    } else {
        println!("{}", bundle.t("status-completed-header", &[]));
        for plan in &status.recent_completed_plans {
            print_plan(bundle, plan);
        }
    }

    if status.recent_completed_tasks.is_empty() {
        println!("{}", bundle.t("status-tasks-empty", &[]));
    } else {
        println!("{}", bundle.t("status-tasks-header", &[]));
        for task in &status.recent_completed_tasks {
            println!(
                "{}",
                bundle.t(
                    "status-task-line",
                    &[
                        ("title", &task.title),
                        ("plan", &task.plan_title),
                        ("project", task.project.as_deref().unwrap_or("-")),
                    ],
                )
            );
        }
    }
}

fn print_plan(bundle: &Bundle, plan: &PlanSummary) {
    println!(
        "{}",
        bundle.t(
            "status-plan-line",
            &[
                ("title", &plan.title),
                ("status", &plan.status),
                ("project", plan.project.as_deref().unwrap_or("-")),
                ("done", &plan.tasks.done.to_string()),
                ("total", &plan.tasks.total.to_string()),
            ],
        )
    );
    let work = plan
        .description
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("-");
    println!("{}", bundle.t("status-work-line", &[("work", work)]));
    let next = plan
        .next_tasks
        .iter()
        .map(|task| task.title.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    println!(
        "{}",
        bundle.t(
            "status-next-line",
            &[("tasks", if next.is_empty() { "-" } else { &next })],
        )
    );
}

#[derive(Debug, Deserialize, Serialize)]
struct StatusResponse {
    active_plans: Vec<PlanSummary>,
    recent_completed_plans: Vec<PlanSummary>,
    recent_completed_tasks: Vec<CompletedTask>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PlanSummary {
    title: String,
    description: Option<String>,
    project: Option<String>,
    status: String,
    tasks: TaskCounts,
    next_tasks: Vec<TaskSummary>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TaskCounts {
    total: usize,
    done: usize,
}

#[derive(Debug, Deserialize, Serialize)]
struct TaskSummary {
    title: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct CompletedTask {
    title: String,
    plan_title: String,
    project: Option<String>,
}

/// Subset of `convergio_durability::AgentRecord` shaped for the CLI.
#[derive(Debug, Deserialize, Serialize)]
struct AgentSummary {
    id: String,
    kind: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    host: Option<String>,
    status: String,
    #[serde(default)]
    current_task_id: Option<String>,
    #[serde(default)]
    last_heartbeat_at: Option<String>,
}
