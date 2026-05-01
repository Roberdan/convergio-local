//! `cvg graph ...` — Tier-3 retrieval client (ADR-0014).
//!
//! Pure HTTP. The daemon owns the SQLite store and the syn parser;
//! the CLI just renders.

use super::{Client, OutputMode};
use anyhow::Result;
use clap::Subcommand;
use serde_json::Value;

/// Graph subcommands.
#[derive(Subcommand)]
pub enum GraphCommand {
    /// Walk the workspace and refresh the graph in the daemon's SQLite.
    Build {
        /// Repo root (defaults to daemon's cwd).
        #[arg(long)]
        manifest_dir: Option<String>,
        /// Re-parse every file even if mtime unchanged.
        #[arg(long)]
        force: bool,
    },
    /// Print the current node + edge counts.
    Stats,
}

/// Entry point.
pub async fn run(client: &Client, output: OutputMode, cmd: GraphCommand) -> Result<()> {
    match cmd {
        GraphCommand::Build {
            manifest_dir,
            force,
        } => build(client, output, manifest_dir, force).await,
        GraphCommand::Stats => stats(client, output).await,
    }
}

async fn build(
    client: &Client,
    output: OutputMode,
    manifest_dir: Option<String>,
    force: bool,
) -> Result<()> {
    let body = serde_json::json!({
        "manifest_dir": manifest_dir,
        "force": force,
    });
    let report: Value = client.post("/v1/graph/build", &body).await?;
    match output {
        OutputMode::Json => println!("{}", serde_json::to_string_pretty(&report)?),
        OutputMode::Plain => render_plain(&report),
        OutputMode::Human => render_human(&report),
    }
    Ok(())
}

async fn stats(client: &Client, output: OutputMode) -> Result<()> {
    let body: Value = client.get("/v1/graph/stats").await?;
    match output {
        OutputMode::Json => println!("{}", serde_json::to_string_pretty(&body)?),
        OutputMode::Plain => render_plain(&body),
        OutputMode::Human => println!(
            "Graph: {} nodes, {} edges.",
            body.get("nodes").and_then(Value::as_u64).unwrap_or(0),
            body.get("edges").and_then(Value::as_u64).unwrap_or(0),
        ),
    }
    Ok(())
}

fn render_plain(v: &Value) {
    println!("{}", serde_json::to_string(v).unwrap_or_default());
}

fn render_human(report: &Value) {
    let nodes = report.get("nodes").and_then(Value::as_u64).unwrap_or(0);
    let edges = report.get("edges").and_then(Value::as_u64).unwrap_or(0);
    let crates = report.get("crates").and_then(Value::as_u64).unwrap_or(0);
    let parsed = report
        .get("files_parsed")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let skipped = report
        .get("files_skipped")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    println!(
        "Graph build: {crates} crates, {parsed} files parsed ({skipped} skipped). \
         Total: {nodes} nodes / {edges} edges."
    );
}
