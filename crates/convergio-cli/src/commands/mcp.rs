//! `cvg mcp ...` — inspect local MCP bridge diagnostics.

use anyhow::{Context, Result};
use clap::Subcommand;
use convergio_i18n::Bundle;
use std::fs;
use std::path::PathBuf;

/// MCP diagnostic subcommands.
#[derive(Subcommand)]
pub enum McpCommand {
    /// Print recent MCP bridge log lines.
    Tail {
        /// Number of recent lines to print.
        #[arg(long, default_value_t = 50)]
        lines: usize,
    },
}

/// Run an MCP diagnostic subcommand.
pub async fn run(bundle: &Bundle, cmd: McpCommand) -> Result<()> {
    match cmd {
        McpCommand::Tail { lines } => tail(bundle, lines),
    }
}

fn tail(bundle: &Bundle, lines: usize) -> Result<()> {
    let path = mcp_log_path()?;
    if !path.exists() {
        println!("{}", bundle.t("mcp-log-missing", &[]));
        return Ok(());
    }
    let content = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    for line in content
        .lines()
        .rev()
        .take(lines)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        println!("{line}");
    }
    Ok(())
}

fn mcp_log_path() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME is not set")?;
    Ok(PathBuf::from(home).join(".convergio/mcp.log"))
}
