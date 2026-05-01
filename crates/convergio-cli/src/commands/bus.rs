//! `cvg bus ...` — human-facing reader (and minimal writer) for the
//! Layer 2 plan-scoped message bus. Agents go through MCP
//! `poll_messages` / `publish_message`; humans land here.
//!
//! All three subcommands (`tail`, `topics`, `post`) accept an optional
//! `--plan <id>` and otherwise resolve the most-recently-updated open
//! plan for `--project <name>` (default `convergio-local`), the same
//! resolver `cvg session resume` uses.

use super::{Client, OutputMode};
use anyhow::{anyhow, Context, Result};
use clap::Subcommand;
use serde::Deserialize;
use serde_json::Value;

/// Bus subcommands.
#[derive(Subcommand)]
pub enum BusCommand {
    /// Print messages on a plan, oldest first. Includes consumed
    /// messages — for unread-only agent-style polling, use the MCP
    /// `poll_messages` action.
    Tail {
        /// Plan id. If omitted, resolves the most recent open plan
        /// in `--project`.
        #[arg(long)]
        plan: Option<String>,
        /// Project filter when no plan id is given.
        #[arg(long, default_value = "convergio-local")]
        project: String,
        /// Optional topic filter. If omitted, every topic is shown.
        #[arg(long)]
        topic: Option<String>,
        /// Only return messages with `seq > since` (exclusive).
        #[arg(long, default_value_t = 0)]
        since: i64,
        /// Cap on the number of messages returned (1..=100).
        #[arg(long, default_value_t = 50)]
        limit: i64,
    },
    /// Print every topic that has at least one message on a plan,
    /// with count + last_seq + last_at.
    Topics {
        #[arg(long)]
        plan: Option<String>,
        #[arg(long, default_value = "convergio-local")]
        project: String,
    },
    /// Publish a JSON payload to a topic. Mostly for ad-hoc human
    /// posts; agents should use the MCP `publish_message` action.
    Post {
        /// Topic to publish on.
        #[arg(long)]
        topic: String,
        /// JSON payload.
        #[arg(long, default_value = "{}")]
        payload: String,
        /// Optional sender (agent id).
        #[arg(long)]
        sender: Option<String>,
        #[arg(long)]
        plan: Option<String>,
        #[arg(long, default_value = "convergio-local")]
        project: String,
    },
}

/// Entry point.
pub async fn run(client: &Client, output: OutputMode, cmd: BusCommand) -> Result<()> {
    match cmd {
        BusCommand::Tail {
            plan,
            project,
            topic,
            since,
            limit,
        } => {
            tail(
                client,
                output,
                plan.as_deref(),
                &project,
                topic.as_deref(),
                since,
                limit,
            )
            .await
        }
        BusCommand::Topics { plan, project } => {
            topics(client, output, plan.as_deref(), &project).await
        }
        BusCommand::Post {
            topic,
            payload,
            sender,
            plan,
            project,
        } => {
            post(
                client,
                output,
                plan.as_deref(),
                &project,
                &topic,
                &payload,
                sender.as_deref(),
            )
            .await
        }
    }
}

async fn tail(
    client: &Client,
    output: OutputMode,
    plan_id: Option<&str>,
    project: &str,
    topic: Option<&str>,
    since: i64,
    limit: i64,
) -> Result<()> {
    let plan = resolve_plan(client, plan_id, project).await?;
    let mut path = format!(
        "/v1/plans/{}/messages/tail?cursor={since}&limit={limit}",
        plan.id
    );
    if let Some(t) = topic {
        path.push_str(&format!("&topic={t}"));
    }
    let messages: Vec<Value> = client.get(&path).await?;
    match output {
        OutputMode::Json => println!("{}", serde_json::to_string_pretty(&messages)?),
        OutputMode::Plain => {
            for m in &messages {
                let seq = m.get("seq").and_then(Value::as_i64).unwrap_or(0);
                let topic = m.get("topic").and_then(Value::as_str).unwrap_or("?");
                let sender = m.get("sender").and_then(Value::as_str).unwrap_or("-");
                println!("seq={seq} topic={topic} sender={sender}");
            }
        }
        OutputMode::Human => render_tail_human(&plan, &messages),
    }
    Ok(())
}

async fn topics(
    client: &Client,
    output: OutputMode,
    plan_id: Option<&str>,
    project: &str,
) -> Result<()> {
    let plan = resolve_plan(client, plan_id, project).await?;
    let summaries: Vec<Value> = client.get(&format!("/v1/plans/{}/topics", plan.id)).await?;
    match output {
        OutputMode::Json => println!("{}", serde_json::to_string_pretty(&summaries)?),
        OutputMode::Plain => {
            for s in &summaries {
                let t = s.get("topic").and_then(Value::as_str).unwrap_or("?");
                let c = s.get("count").and_then(Value::as_i64).unwrap_or(0);
                let last = s.get("last_seq").and_then(Value::as_i64).unwrap_or(0);
                println!("topic={t} count={c} last_seq={last}");
            }
        }
        OutputMode::Human => {
            println!("Plan {} ({} topics)", plan.id, summaries.len());
            for s in &summaries {
                let t = s.get("topic").and_then(Value::as_str).unwrap_or("?");
                let c = s.get("count").and_then(Value::as_i64).unwrap_or(0);
                let last = s.get("last_seq").and_then(Value::as_i64).unwrap_or(0);
                let at = s.get("last_at").and_then(Value::as_str).unwrap_or("?");
                println!("  - {t} ({c} msgs, last seq={last} at {at})");
            }
        }
    }
    Ok(())
}

async fn post(
    client: &Client,
    output: OutputMode,
    plan_id: Option<&str>,
    project: &str,
    topic: &str,
    payload: &str,
    sender: Option<&str>,
) -> Result<()> {
    let plan = resolve_plan(client, plan_id, project).await?;
    let payload: Value = serde_json::from_str(payload)
        .with_context(|| format!("payload must be valid JSON: {payload}"))?;
    let body = serde_json::json!({
        "topic": topic,
        "payload": payload,
        "sender": sender,
    });
    let m: Value = client
        .post(&format!("/v1/plans/{}/messages", plan.id), &body)
        .await?;
    match output {
        OutputMode::Json => println!("{}", serde_json::to_string_pretty(&m)?),
        OutputMode::Plain => {
            let seq = m.get("seq").and_then(Value::as_i64).unwrap_or(0);
            let id = m.get("id").and_then(Value::as_str).unwrap_or("?");
            println!("seq={seq} id={id}");
        }
        OutputMode::Human => {
            let seq = m.get("seq").and_then(Value::as_i64).unwrap_or(0);
            println!("Posted to {topic} on plan {} as seq {seq}.", plan.id);
        }
    }
    Ok(())
}

fn render_tail_human(plan: &Plan, messages: &[Value]) {
    println!("Plan {} — {} message(s)", plan.id, messages.len());
    for m in messages {
        let seq = m.get("seq").and_then(Value::as_i64).unwrap_or(0);
        let topic = m.get("topic").and_then(Value::as_str).unwrap_or("?");
        let sender = m.get("sender").and_then(Value::as_str).unwrap_or("-");
        let consumed = m.get("consumed_at").and_then(Value::as_str).is_some();
        let kind = m
            .get("payload")
            .and_then(|p| p.get("kind"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let mark = if consumed { " [acked]" } else { "" };
        let kind_part = if kind.is_empty() {
            String::new()
        } else {
            format!(" {kind}")
        };
        println!("  seq {seq:>3} [{topic}] sender={sender}{kind_part}{mark}");
    }
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
        .filter(|p| matches!(p.status.as_str(), "draft" | "active"))
        .max_by(|a, b| a.updated_at.cmp(&b.updated_at))
        .ok_or_else(|| anyhow!("no open plan found for project={project}"))
}

#[derive(Debug, Deserialize)]
struct Plan {
    id: String,
    #[serde(default)]
    project: Option<String>,
    status: String,
    updated_at: String,
}
