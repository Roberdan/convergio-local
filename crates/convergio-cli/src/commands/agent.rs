//! `cvg agent ...` — surface the durable agent registry as a CLI
//! query.
//!
//! Closes the F46 half-wired bit (F55 in friction log): the daemon
//! sync of `agents.current_task_id` was already in main, but the
//! only way to observe it was direct sqlite SELECT. This command
//! turns the live state query into a first-class human/JSON/plain
//! surface.

use super::{Client, OutputMode};
use anyhow::Result;
use clap::Subcommand;
use convergio_i18n::Bundle;
use serde_json::Value;

/// Agent registry subcommands.
#[derive(Subcommand)]
pub enum AgentCommand {
    /// List all registered agents and their live status.
    List,
    /// Show a single agent record by id.
    Show {
        /// Agent id (e.g. `claude-code-roberdan`).
        id: String,
    },
}

/// Dispatch.
pub async fn run(
    client: &Client,
    bundle: &Bundle,
    output: OutputMode,
    cmd: AgentCommand,
) -> Result<()> {
    match cmd {
        AgentCommand::List => list(client, bundle, output).await,
        AgentCommand::Show { id } => show(client, bundle, output, &id).await,
    }
}

async fn list(client: &Client, bundle: &Bundle, output: OutputMode) -> Result<()> {
    let agents: Value = client.get("/v1/agent-registry/agents").await?;
    let count = agents.as_array().map(|a| a.len() as i64).unwrap_or(0);
    match output {
        OutputMode::Human => render_human_list(bundle, &agents, count),
        OutputMode::Json => println!("{}", serde_json::to_string_pretty(&agents)?),
        OutputMode::Plain => render_plain_list(&agents),
    }
    Ok(())
}

fn render_human_list(bundle: &Bundle, agents: &Value, count: i64) {
    if count == 0 {
        println!("{}", bundle.t("agent-list-empty", &[]));
        return;
    }
    println!("{}", bundle.t_n("agent-list-header", count));
    println!(
        "{:<28} {:<18} {:<10} {:<36}",
        bundle.t("agent-list-col-id", &[]),
        bundle.t("agent-list-col-kind", &[]),
        bundle.t("agent-list-col-status", &[]),
        bundle.t("agent-list-col-current-task", &[]),
    );
    if let Some(arr) = agents.as_array() {
        for a in arr {
            let id = field(a, "id");
            let kind = field(a, "kind");
            let status = field(a, "status");
            let current = a
                .get("current_task_id")
                .and_then(Value::as_str)
                .unwrap_or("-");
            println!("{id:<28} {kind:<18} {status:<10} {current:<36}");
        }
    }
}

fn render_plain_list(agents: &Value) {
    if let Some(arr) = agents.as_array() {
        for a in arr {
            let id = field(a, "id");
            let status = field(a, "status");
            let current = a
                .get("current_task_id")
                .and_then(Value::as_str)
                .unwrap_or("-");
            println!("{id}\t{status}\t{current}");
        }
    }
}

async fn show(client: &Client, bundle: &Bundle, output: OutputMode, id: &str) -> Result<()> {
    match client
        .get::<Value>(&format!("/v1/agent-registry/agents/{id}"))
        .await
    {
        Ok(agent) => {
            match output {
                OutputMode::Human => render_human_show(bundle, &agent),
                OutputMode::Json => println!("{}", serde_json::to_string_pretty(&agent)?),
                OutputMode::Plain => {
                    println!(
                        "{}\t{}\t{}",
                        field(&agent, "id"),
                        field(&agent, "status"),
                        agent
                            .get("current_task_id")
                            .and_then(Value::as_str)
                            .unwrap_or("-"),
                    );
                }
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("{}", bundle.t("agent-not-found", &[("id", id)]));
            Err(e)
        }
    }
}

fn render_human_show(bundle: &Bundle, agent: &Value) {
    println!(
        "{}",
        bundle.t("agent-show-header", &[("id", field(agent, "id"))])
    );
    println!("  kind:             {}", field(agent, "kind"));
    println!("  status:           {}", field(agent, "status"));
    let current = agent
        .get("current_task_id")
        .and_then(Value::as_str)
        .unwrap_or("-");
    println!("  current_task_id:  {current}");
    if let Some(host) = agent.get("host").and_then(Value::as_str) {
        println!("  host:             {host}");
    }
    if let Some(name) = agent.get("name").and_then(Value::as_str) {
        println!("  name:             {name}");
    }
    if let Some(caps) = agent.get("capabilities").and_then(Value::as_array) {
        let joined = caps
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        if !joined.is_empty() {
            println!("  capabilities:     {joined}");
        }
    }
    if let Some(hb) = agent.get("last_heartbeat_at").and_then(Value::as_str) {
        println!("  last_heartbeat:   {hb}");
    }
}

fn field<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("?")
}
