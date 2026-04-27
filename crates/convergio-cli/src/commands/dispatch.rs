//! `cvg dispatch` — run one executor tick.

use super::Client;
use anyhow::Result;
use serde_json::{json, Value};

/// Run the command.
pub async fn run(client: &Client) -> Result<()> {
    let body: Value = client.post("/v1/dispatch", &json!({})).await?;
    println!("{}", serde_json::to_string_pretty(&body)?);
    Ok(())
}
