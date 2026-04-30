//! `cvg plan ...` — create / list / get plans.

use super::{Client, OutputMode};
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
        /// Optional project or repository this plan belongs to.
        #[arg(long)]
        project: Option<String>,
    },
    /// List plans.
    List {
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
pub async fn run(
    client: &Client,
    bundle: &Bundle,
    output: OutputMode,
    cmd: PlanCommand,
) -> Result<()> {
    match cmd {
        PlanCommand::Create {
            title,
            description,
            project,
        } => {
            let body = json!({
                "title": title,
                "description": description,
                "project": project,
            });
            let plan: Value = client.post("/v1/plans", &body).await?;
            let id = plan.get("id").and_then(Value::as_str).unwrap_or("?");
            match output {
                OutputMode::Human => {
                    println!("{}", bundle.t("plan-created", &[("id", id)]));
                }
                OutputMode::Json => {
                    println!("{}", serde_json::to_string_pretty(&plan)?);
                }
                OutputMode::Plain => {
                    println!("{id}");
                }
            }
        }
        PlanCommand::List { limit } => {
            let path = format!("/v1/plans?limit={limit}");
            let plans: Value = client.get(&path).await?;
            let count = plans.as_array().map(|a| a.len() as i64).unwrap_or(0);
            match output {
                OutputMode::Human => {
                    if count == 0 {
                        println!("{}", bundle.t("plan-list-empty", &[]));
                    } else {
                        println!("{}", bundle.t_n("plan-list-header", count));
                        println!("{}", serde_json::to_string_pretty(&plans)?);
                    }
                }
                OutputMode::Json => {
                    println!("{}", serde_json::to_string_pretty(&plans)?);
                }
                OutputMode::Plain => {
                    if let Some(arr) = plans.as_array() {
                        for plan in arr {
                            if let Some(id) = plan.get("id").and_then(Value::as_str) {
                                println!("{id}");
                            }
                        }
                    }
                }
            }
        }
        PlanCommand::Get { id } => match client.get::<Value>(&format!("/v1/plans/{id}")).await {
            Ok(plan) => match output {
                OutputMode::Human | OutputMode::Json => {
                    println!("{}", serde_json::to_string_pretty(&plan)?);
                }
                OutputMode::Plain => {
                    if let Some(plan_id) = plan.get("id").and_then(Value::as_str) {
                        println!("{plan_id}");
                    }
                }
            },
            Err(e) => {
                eprintln!("{}", bundle.t("plan-not-found", &[("id", &id)]));
                return Err(e);
            }
        },
    }
    Ok(())
}
