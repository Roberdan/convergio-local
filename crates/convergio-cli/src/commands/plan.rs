//! `cvg plan ...` — create / list / get plans.

use super::Client;
use anyhow::Result;
use clap::Subcommand;
use serde_json::{json, Value};

/// Plan subcommands.
#[derive(Subcommand)]
pub enum PlanCommand {
    /// Create a new plan.
    Create {
        /// Plan title.
        title: String,
        /// Optional description.
        #[arg(long)]
        description: Option<String>,
        /// Org id (defaults to `default`).
        #[arg(long, default_value = "default")]
        org_id: String,
    },
    /// List plans.
    List {
        /// Org id.
        #[arg(long, default_value = "default")]
        org_id: String,
        /// Max rows to return.
        #[arg(long, default_value_t = 50)]
        limit: i64,
    },
    /// Get a plan by id.
    Get {
        /// UUID of the plan.
        id: String,
    },
}

/// Dispatch.
pub async fn run(client: &Client, cmd: PlanCommand) -> Result<()> {
    match cmd {
        PlanCommand::Create {
            title,
            description,
            org_id,
        } => {
            let body = json!({
                "title": title,
                "description": description,
                "org_id": org_id,
            });
            let plan: Value = client.post("/v1/plans", &body).await?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        PlanCommand::List { org_id, limit } => {
            let path = format!("/v1/plans?org_id={org_id}&limit={limit}");
            let plans: Value = client.get(&path).await?;
            println!("{}", serde_json::to_string_pretty(&plans)?);
        }
        PlanCommand::Get { id } => {
            let plan: Value = client.get(&format!("/v1/plans/{id}")).await?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
    }
    Ok(())
}
