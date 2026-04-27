//! `cvg audit ...` — verify the chain.

use super::Client;
use anyhow::Result;
use clap::Subcommand;
use serde_json::Value;

/// Audit subcommands.
#[derive(Subcommand)]
pub enum AuditCommand {
    /// Recompute and verify the hash chain.
    Verify {
        /// Lower bound (inclusive) on `seq`.
        #[arg(long)]
        from: Option<i64>,
        /// Upper bound (inclusive) on `seq`.
        #[arg(long)]
        to: Option<i64>,
    },
}

/// Dispatch.
pub async fn run(client: &Client, cmd: AuditCommand) -> Result<()> {
    let AuditCommand::Verify { from, to } = cmd;
    let mut path = String::from("/v1/audit/verify?");
    if let Some(f) = from {
        path.push_str(&format!("from={f}&"));
    }
    if let Some(t) = to {
        path.push_str(&format!("to={t}&"));
    }
    let report: Value = client.get(&path).await?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
