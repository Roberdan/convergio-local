//! `cvg health` — probe the daemon.

use super::Client;
use anyhow::Result;
use serde_json::Value;

/// Run the command.
pub async fn run(client: &Client) -> Result<()> {
    let body: Value = client.get("/v1/health").await?;
    println!("{}", serde_json::to_string_pretty(&body)?);
    Ok(())
}
