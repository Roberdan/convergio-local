//! `cvg workspace` — workspace coordination diagnostics.

use super::{Client, OutputMode};
use anyhow::Result;
use clap::Subcommand;
use convergio_i18n::Bundle;
use serde::Deserialize;
use serde_json::Value;

/// Workspace subcommands.
#[derive(Subcommand)]
pub enum WorkspaceCommand {
    /// List active workspace leases.
    Leases,
}

/// Run a workspace diagnostic subcommand.
pub async fn run(
    client: &Client,
    bundle: &Bundle,
    output: OutputMode,
    sub: WorkspaceCommand,
) -> Result<()> {
    match sub {
        WorkspaceCommand::Leases => leases(client, bundle, output).await,
    }
}

async fn leases(client: &Client, bundle: &Bundle, output: OutputMode) -> Result<()> {
    let body: Value = client.get("/v1/workspace/leases").await?;
    match output {
        OutputMode::Json => println!("{}", serde_json::to_string_pretty(&body)?),
        OutputMode::Plain => {
            let leases: Vec<WorkspaceLease> = serde_json::from_value(body)?;
            println!("workspace_leases={}", leases.len());
        }
        OutputMode::Human => {
            let leases: Vec<WorkspaceLease> = serde_json::from_value(body)?;
            render_human(bundle, &leases);
        }
    }
    Ok(())
}

fn render_human(bundle: &Bundle, leases: &[WorkspaceLease]) {
    if leases.is_empty() {
        println!("{}", bundle.t("workspace-leases-empty", &[]));
        return;
    }
    println!("{}", bundle.t("workspace-leases-header", &[]));
    for lease in leases {
        println!(
            "{}",
            bundle.t(
                "workspace-lease-line",
                &[
                    ("agent", &lease.agent_id),
                    ("kind", &lease.resource.kind),
                    ("path", &lease.resource.path),
                    ("expires", &lease.expires_at),
                ],
            )
        );
    }
}

#[derive(Debug, Deserialize)]
struct WorkspaceLease {
    agent_id: String,
    expires_at: String,
    resource: WorkspaceResource,
}

#[derive(Debug, Deserialize)]
struct WorkspaceResource {
    kind: String,
    path: String,
}
