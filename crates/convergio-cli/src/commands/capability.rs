//! `cvg capability` — local capability registry diagnostics.

use super::{Client, OutputMode};
use anyhow::Result;
use clap::Subcommand;
use convergio_i18n::Bundle;
use serde::Deserialize;
use serde_json::Value;

/// Capability subcommands.
#[derive(Subcommand)]
pub enum CapabilityCommand {
    /// List local capability registry rows.
    List,
}

/// Run a capability registry subcommand.
pub async fn run(
    client: &Client,
    bundle: &Bundle,
    output: OutputMode,
    sub: CapabilityCommand,
) -> Result<()> {
    match sub {
        CapabilityCommand::List => list(client, bundle, output).await,
    }
}

async fn list(client: &Client, bundle: &Bundle, output: OutputMode) -> Result<()> {
    let body: Value = client.get("/v1/capabilities").await?;
    match output {
        OutputMode::Json => println!("{}", serde_json::to_string_pretty(&body)?),
        OutputMode::Plain => {
            let caps: Vec<Capability> = serde_json::from_value(body)?;
            println!("capabilities={}", caps.len());
        }
        OutputMode::Human => {
            let caps: Vec<Capability> = serde_json::from_value(body)?;
            render_human(bundle, &caps);
        }
    }
    Ok(())
}

fn render_human(bundle: &Bundle, caps: &[Capability]) {
    if caps.is_empty() {
        println!("{}", bundle.t("capabilities-empty", &[]));
        return;
    }
    println!("{}", bundle.t("capabilities-header", &[]));
    for cap in caps {
        println!(
            "{}",
            bundle.t(
                "capability-line",
                &[
                    ("name", &cap.name),
                    ("version", &cap.version),
                    ("status", &cap.status),
                ],
            )
        );
    }
}

#[derive(Debug, Deserialize)]
struct Capability {
    name: String,
    version: String,
    status: String,
}
