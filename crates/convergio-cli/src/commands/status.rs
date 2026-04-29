//! `cvg status` — local dashboard for plans and recently completed work.

use super::{Client, OutputMode};
use anyhow::Result;
use convergio_i18n::Bundle;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Run `cvg status`.
pub async fn run(
    client: &Client,
    bundle: &Bundle,
    output: OutputMode,
    completed_limit: i64,
) -> Result<()> {
    let path = format!("/v1/status?completed_limit={completed_limit}");
    let body: Value = client.get(&path).await?;
    match output {
        OutputMode::Json => println!("{}", serde_json::to_string_pretty(&body)?),
        OutputMode::Plain => render_plain(&serde_json::from_value(body)?),
        OutputMode::Human => render_human(bundle, &serde_json::from_value(body)?),
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
