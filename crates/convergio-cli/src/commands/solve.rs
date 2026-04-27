//! `cvg solve <mission>` — turn a mission into a plan.

use super::Client;
use anyhow::Result;
use serde_json::{json, Value};

/// Run the command.
pub async fn run(client: &Client, mission: &str) -> Result<()> {
    let body: Value = client
        .post("/v1/solve", &json!({"mission": mission}))
        .await?;
    println!("{}", serde_json::to_string_pretty(&body)?);
    Ok(())
}
