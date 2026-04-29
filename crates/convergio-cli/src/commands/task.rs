//! `cvg task ...` — inspect and transition local tasks.

use super::Client;
use anyhow::{bail, Result};
use clap::{Subcommand, ValueEnum};
use serde_json::{json, Value};

/// Task subcommands.
#[derive(Subcommand)]
pub enum TaskCommand {
    /// List tasks of a plan.
    List {
        /// Plan id.
        plan_id: String,
    },
    /// Get one task.
    Get {
        /// Task id.
        task_id: String,
    },
    /// Move a task to a new status, running server-side gates.
    Transition {
        /// Task id.
        task_id: String,
        /// Target status.
        target: TaskTarget,
        /// Agent id to record on the transition.
        #[arg(long)]
        agent_id: Option<String>,
    },
    /// Touch a task heartbeat.
    Heartbeat {
        /// Task id.
        task_id: String,
    },
}

/// CLI-friendly task status values.
#[derive(Clone, Copy, ValueEnum)]
pub enum TaskTarget {
    /// Claimed and being worked on.
    InProgress,
    /// Agent claims completion; awaiting validation.
    Submitted,
    /// Validated and closed.
    Done,
    /// Failed and not retryable.
    Failed,
    /// Release back to pending.
    Pending,
}

impl TaskTarget {
    fn as_api(self) -> &'static str {
        match self {
            Self::InProgress => "in_progress",
            Self::Submitted => "submitted",
            Self::Done => "done",
            Self::Failed => "failed",
            Self::Pending => "pending",
        }
    }
}

/// Run a task subcommand.
pub async fn run(client: &Client, cmd: TaskCommand) -> Result<()> {
    match cmd {
        TaskCommand::List { plan_id } => {
            let tasks: Value = client.get(&format!("/v1/plans/{plan_id}/tasks")).await?;
            print_json(&tasks)?;
        }
        TaskCommand::Get { task_id } => {
            let task: Value = client.get(&format!("/v1/tasks/{task_id}")).await?;
            print_json(&task)?;
        }
        TaskCommand::Transition {
            task_id,
            target,
            agent_id,
        } => {
            let body = json!({
                "target": target.as_api(),
                "agent_id": agent_id,
            });
            let task: Value = client
                .post(&format!("/v1/tasks/{task_id}/transition"), &body)
                .await?;
            print_json(&task)?;
        }
        TaskCommand::Heartbeat { task_id } => {
            let ok: Value = client
                .post(&format!("/v1/tasks/{task_id}/heartbeat"), &json!({}))
                .await?;
            if ok.get("ok").and_then(Value::as_bool) != Some(true) {
                bail!("unexpected heartbeat response: {ok}");
            }
            print_json(&ok)?;
        }
    }
    Ok(())
}

fn print_json(value: &Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
