//! `cvg evidence ...` — attach and inspect task evidence.

use super::Client;
use anyhow::{Context, Result};
use clap::Subcommand;
use serde_json::{json, Value};

/// Evidence subcommands.
#[derive(Subcommand)]
pub enum EvidenceCommand {
    /// Attach evidence to a task.
    Add {
        /// Task id.
        task_id: String,
        /// Evidence kind (`code`, `test`, `lint`, ...).
        #[arg(long)]
        kind: String,
        /// JSON payload.
        #[arg(long, default_value = "{}")]
        payload: String,
        /// Optional process exit code.
        #[arg(long)]
        exit_code: Option<i64>,
    },
    /// List evidence rows for a task.
    List {
        /// Task id.
        task_id: String,
    },
}

/// Run an evidence subcommand.
pub async fn run(client: &Client, cmd: EvidenceCommand) -> Result<()> {
    match cmd {
        EvidenceCommand::Add {
            task_id,
            kind,
            payload,
            exit_code,
        } => {
            let payload: Value = serde_json::from_str(&payload)
                .with_context(|| format!("payload must be valid JSON: {payload}"))?;
            let body = json!({
                "kind": kind,
                "payload": payload,
                "exit_code": exit_code,
            });
            let evidence: Value = client
                .post(&format!("/v1/tasks/{task_id}/evidence"), &body)
                .await?;
            print_json(&evidence)?;
        }
        EvidenceCommand::List { task_id } => {
            let evidence: Value = client.get(&format!("/v1/tasks/{task_id}/evidence")).await?;
            print_json(&evidence)?;
        }
    }
    Ok(())
}

fn print_json(value: &Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
