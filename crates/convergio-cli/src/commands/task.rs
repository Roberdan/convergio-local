//! `cvg task ...` — create, inspect and transition local tasks.

use super::task_render::{render_task, render_task_list};
use super::{Client, OutputMode};
use anyhow::{bail, Result};
use clap::{Subcommand, ValueEnum};
use serde_json::{json, Value};

/// Task subcommands.
#[derive(Subcommand)]
pub enum TaskCommand {
    /// Create a task under a plan.
    Create {
        /// Owning plan id.
        plan_id: String,
        /// Task title (short).
        title: String,
        /// Optional long description.
        #[arg(long)]
        description: Option<String>,
        /// Wave number (defaults to 1).
        #[arg(long, default_value_t = 1)]
        wave: i64,
        /// Sequence within the wave (defaults to 1).
        #[arg(long, default_value_t = 1)]
        sequence: i64,
        /// Required evidence kinds (comma-separated, e.g. `code,test`).
        #[arg(long, value_delimiter = ',')]
        evidence_required: Vec<String>,
    },
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

/// CLI-friendly task status values that an agent may request.
///
/// `done` is intentionally absent: it is set only by the validator
/// (`cvg validate <plan_id>`). See CONSTITUTION §6 and ADR-0011.
#[derive(Clone, Copy, ValueEnum)]
pub enum TaskTarget {
    /// Claimed and being worked on.
    InProgress,
    /// Agent claims completion; awaiting validation.
    Submitted,
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
            Self::Failed => "failed",
            Self::Pending => "pending",
        }
    }
}

/// Run a task subcommand.
pub async fn run(client: &Client, output: OutputMode, cmd: TaskCommand) -> Result<()> {
    match cmd {
        TaskCommand::Create {
            plan_id,
            title,
            description,
            wave,
            sequence,
            evidence_required,
        } => {
            let body = json!({
                "title": title,
                "description": description,
                "wave": wave,
                "sequence": sequence,
                "evidence_required": evidence_required,
            });
            let task: Value = client
                .post(&format!("/v1/plans/{plan_id}/tasks"), &body)
                .await?;
            render_task(&task, output)
        }
        TaskCommand::List { plan_id } => {
            let tasks: Value = client.get(&format!("/v1/plans/{plan_id}/tasks")).await?;
            render_task_list(&tasks, output)
        }
        TaskCommand::Get { task_id } => {
            let task: Value = client.get(&format!("/v1/tasks/{task_id}")).await?;
            render_task(&task, output)
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
            render_task(&task, output)
        }
        TaskCommand::Heartbeat { task_id } => {
            let ok: Value = client
                .post(&format!("/v1/tasks/{task_id}/heartbeat"), &json!({}))
                .await?;
            if ok.get("ok").and_then(Value::as_bool) != Some(true) {
                bail!("unexpected heartbeat response: {ok}");
            }
            match output {
                OutputMode::Plain => println!("ok"),
                _ => println!("{}", serde_json::to_string_pretty(&ok)?),
            }
            Ok(())
        }
    }
}
