//! `cvg plan ...` — create / list / get plans.

use super::Client;
use anyhow::Result;
use clap::Subcommand;
use convergio_i18n::Bundle;
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
pub async fn run(client: &Client, bundle: &Bundle, cmd: PlanCommand) -> Result<()> {
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
            let id = plan.get("id").and_then(Value::as_str).unwrap_or("?");
            println!("{}", bundle.t("plan-created", &[("id", id)]));
            // Also dump the JSON for scripts that need it.
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        PlanCommand::List { org_id, limit } => {
            let path = format!("/v1/plans?org_id={org_id}&limit={limit}");
            let plans: Value = client.get(&path).await?;
            let count = plans.as_array().map(|a| a.len() as i64).unwrap_or(0);
            if count == 0 {
                println!("{}", bundle.t("plan-list-empty", &[]));
            } else {
                println!("{}", bundle.t_n("plan-list-header", count));
                println!("{}", serde_json::to_string_pretty(&plans)?);
            }
        }
        PlanCommand::Get { id } => match client.get::<Value>(&format!("/v1/plans/{id}")).await {
            Ok(plan) => {
                println!("{}", serde_json::to_string_pretty(&plan)?);
            }
            Err(e) => {
                eprintln!("{}", bundle.t("plan-not-found", &[("id", &id)]));
                return Err(e);
            }
        },
    }
    Ok(())
}
