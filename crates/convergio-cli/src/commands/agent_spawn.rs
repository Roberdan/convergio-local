//! `cvg agent spawn` — start a vendor-CLI agent against a task.
//!
//! Pulls the task + plan + (optionally) the graph context-pack from
//! the daemon, hands them to the right [`convergio_runner::Runner`],
//! then either prints the prepared command (`--dry-run`) or executes
//! it inline. The vendor CLI inherits the operator's existing auth;
//! Convergio never sees the API key (ADR-0032).

use super::{Client, OutputMode};
use anyhow::{anyhow, Context as _, Result};
use chrono::{DateTime, Utc};
use convergio_durability::{Task, TaskStatus};
use convergio_runner::{for_kind, RunnerKind, SpawnContext};
use serde::Deserialize;
use serde_json::Value;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

/// All operator-controlled flags from `cvg agent spawn`. Bundling
/// them in one struct keeps the dispatcher under clippy's
/// `too_many_arguments` lint and makes the call-site readable.
pub struct SpawnArgs {
    /// `--task` value.
    pub task_id: String,
    /// `--runner` wire string.
    pub runner: String,
    /// Override stable agent id (default derived from kind+task).
    pub agent_id: Option<String>,
    /// Override working directory (default cwd).
    pub cwd: Option<PathBuf>,
    /// `--max-budget-usd` (Claude only).
    pub max_budget_usd: Option<f32>,
    /// `--dry-run` toggle.
    pub dry_run: bool,
}

/// Run `cvg agent spawn`.
pub async fn run(client: &Client, output: OutputMode, args: SpawnArgs) -> Result<()> {
    let SpawnArgs {
        task_id,
        runner,
        agent_id,
        cwd,
        max_budget_usd,
        dry_run,
    } = args;
    let kind = RunnerKind::from_str(&runner).context("parse --runner")?;
    let task = fetch_task(client, &task_id).await?;
    let plan = fetch_plan(client, &task.plan_id).await?;
    let plan_title = plan
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("(untitled)");

    let graph_context = fetch_graph_context(client, &task_id).await;
    let graph_context_ref = graph_context.as_deref();

    let agent = agent_id
        .clone()
        .unwrap_or_else(|| default_agent_id(&kind, &task.id));
    let cwd_ref = cwd
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let ctx = SpawnContext {
        task: &task,
        plan_id: &task.plan_id,
        plan_title,
        daemon_url: client.base(),
        agent_id: &agent,
        graph_context: graph_context_ref,
        cwd: &cwd_ref,
        max_budget_usd,
    };
    let prepared = for_kind(&kind).prepare(&ctx)?;

    if dry_run {
        emit_dry_run(output, &prepared, &kind, &agent)?;
        return Ok(());
    }

    let (mut cmd, prompt) = prepared.into_std_command();
    let summary = format!(
        "spawning {kind} for task {} (agent={agent}, cwd={}) ...",
        task.id,
        cwd_ref.display(),
    );
    if matches!(output, OutputMode::Human | OutputMode::Plain) {
        eprintln!("{summary}");
    }

    let mut child = cmd.spawn().with_context(|| {
        format!(
            "failed to spawn vendor CLI `{}` — is it installed and on PATH?",
            kind.family.cli()
        )
    })?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(prompt.as_bytes())
            .context("write prompt to vendor CLI stdin")?;
    }
    let status = child.wait().context("wait for vendor CLI to exit")?;
    if !status.success() {
        return Err(anyhow!(
            "vendor CLI exited with non-zero status: {:?}",
            status.code()
        ));
    }
    Ok(())
}

fn emit_dry_run(
    output: OutputMode,
    prepared: &convergio_runner::PreparedCommand,
    kind: &RunnerKind,
    agent: &str,
) -> Result<()> {
    match output {
        OutputMode::Json => {
            let argv: Vec<String> = prepared
                .args
                .iter()
                .map(|a| a.to_string_lossy().into_owned())
                .collect();
            let v = serde_json::json!({
                "kind": kind.to_string(),
                "agent_id": agent,
                "program": prepared.program.to_string_lossy(),
                "args": argv,
                "cwd": prepared.cwd.display().to_string(),
                "stdin_prompt_bytes": prepared.stdin_prompt.len(),
                "stdin_prompt": prepared.stdin_prompt,
            });
            println!("{}", serde_json::to_string_pretty(&v)?);
        }
        _ => {
            println!("# cvg agent spawn — dry run");
            println!("kind:  {kind}");
            println!("agent: {agent}");
            println!("cwd:   {}", prepared.cwd.display());
            print!("argv:  {}", prepared.program.to_string_lossy());
            for a in &prepared.args {
                print!(" {}", a.to_string_lossy());
            }
            println!();
            println!("\n--- prompt ({} bytes) ---", prepared.stdin_prompt.len());
            println!("{}", prepared.stdin_prompt);
        }
    }
    Ok(())
}

/// Build a default agent id like `claude-sonnet-t1234abc`.
fn default_agent_id(kind: &RunnerKind, task_id: &str) -> String {
    let slug = task_id.get(..7).unwrap_or(task_id);
    format!("{}-{}-{}", kind.family.tag(), kind.model, slug)
}

async fn fetch_task(client: &Client, id: &str) -> Result<Task> {
    let raw: Value = client.get(&format!("/v1/tasks/{id}")).await?;
    let row: TaskWire =
        serde_json::from_value(raw.clone()).context("decode task from /v1/tasks")?;
    Ok(row.into_task())
}

async fn fetch_plan(client: &Client, id: &str) -> Result<Value> {
    client.get(&format!("/v1/plans/{id}")).await
}

/// Best-effort: returns `None` when the graph is not built or the
/// route fails. The runner copes with absent context.
async fn fetch_graph_context(client: &Client, task_id: &str) -> Option<String> {
    let raw: Value = client
        .get(&format!("/v1/graph/for-task/{task_id}"))
        .await
        .ok()?;
    raw.get("text")
        .and_then(Value::as_str)
        .map(|s| s.to_string())
        .or_else(|| serde_json::to_string_pretty(&raw).ok())
}

/// Wire shape returned by `GET /v1/tasks/:id`. Mirrors
/// [`convergio_durability::Task`] but accepts string statuses
/// (the daemon serialises the enum as snake_case).
#[derive(Deserialize)]
struct TaskWire {
    id: String,
    plan_id: String,
    wave: i64,
    sequence: i64,
    title: String,
    description: Option<String>,
    status: String,
    agent_id: Option<String>,
    #[serde(default)]
    evidence_required: Vec<String>,
    last_heartbeat_at: Option<String>,
    created_at: String,
    updated_at: String,
    #[serde(default)]
    started_at: Option<String>,
    #[serde(default)]
    ended_at: Option<String>,
    #[serde(default)]
    duration_ms: Option<i64>,
}

impl TaskWire {
    fn into_task(self) -> Task {
        Task {
            id: self.id,
            plan_id: self.plan_id,
            wave: self.wave,
            sequence: self.sequence,
            title: self.title,
            description: self.description,
            status: TaskStatus::parse(&self.status).unwrap_or(TaskStatus::Pending),
            agent_id: self.agent_id,
            evidence_required: self.evidence_required,
            last_heartbeat_at: parse_ts_opt(self.last_heartbeat_at.as_deref()),
            created_at: parse_ts(&self.created_at).unwrap_or_else(Utc::now),
            updated_at: parse_ts(&self.updated_at).unwrap_or_else(Utc::now),
            started_at: parse_ts_opt(self.started_at.as_deref()),
            ended_at: parse_ts_opt(self.ended_at.as_deref()),
            duration_ms: self.duration_ms,
        }
    }
}

fn parse_ts(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|t| t.with_timezone(&Utc))
}

fn parse_ts_opt(s: Option<&str>) -> Option<DateTime<Utc>> {
    s.and_then(parse_ts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_agent_id_uses_kind_and_slug() {
        let kind = RunnerKind::claude_sonnet();
        let id = default_agent_id(&kind, "abcdef1234567890");
        assert_eq!(id, "claude-sonnet-abcdef1");
    }

    #[test]
    fn default_agent_id_handles_short_task_id() {
        let kind = RunnerKind::copilot_gpt();
        let id = default_agent_id(&kind, "abc");
        assert_eq!(id, "copilot-gpt-5.2-abc");
    }
}
