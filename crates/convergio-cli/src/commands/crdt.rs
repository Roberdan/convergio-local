//! `cvg crdt` — CRDT diagnostics.

use super::{Client, OutputMode};
use anyhow::Result;
use clap::Subcommand;
use convergio_i18n::Bundle;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// CRDT subcommands.
#[derive(Subcommand)]
pub enum CrdtCommand {
    /// List unresolved CRDT conflicts.
    Conflicts,
}

/// Run a CRDT diagnostic subcommand.
pub async fn run(
    client: &Client,
    bundle: &Bundle,
    output: OutputMode,
    sub: CrdtCommand,
) -> Result<()> {
    match sub {
        CrdtCommand::Conflicts => conflicts(client, bundle, output).await,
    }
}

async fn conflicts(client: &Client, bundle: &Bundle, output: OutputMode) -> Result<()> {
    let body: Value = client.get("/v1/crdt/conflicts").await?;
    match output {
        OutputMode::Json => println!("{}", serde_json::to_string_pretty(&body)?),
        OutputMode::Plain => {
            let conflicts: Vec<CrdtCell> = serde_json::from_value(body)?;
            println!("crdt_conflicts={}", conflicts.len());
        }
        OutputMode::Human => {
            let conflicts: Vec<CrdtCell> = serde_json::from_value(body)?;
            render_human(bundle, &conflicts);
        }
    }
    Ok(())
}

fn render_human(bundle: &Bundle, conflicts: &[CrdtCell]) {
    if conflicts.is_empty() {
        println!("{}", bundle.t("crdt-conflicts-empty", &[]));
        return;
    }

    println!("{}", bundle.t("crdt-conflicts-header", &[]));
    for conflict in conflicts {
        println!(
            "{}",
            bundle.t(
                "crdt-conflict-line",
                &[
                    ("entity", &conflict.entity_type),
                    ("id", &conflict.entity_id),
                    ("field", &conflict.field_name),
                    ("type", &conflict.crdt_type),
                ],
            )
        );
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct CrdtCell {
    entity_type: String,
    entity_id: String,
    field_name: String,
    crdt_type: String,
}
