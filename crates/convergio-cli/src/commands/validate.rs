//! `cvg validate <plan_id>` — Thor verdict.

use super::Client;
use anyhow::Result;
use serde_json::{json, Value};

/// Run the command.
pub async fn run(client: &Client, plan_id: &str) -> Result<()> {
    let body: Value = client
        .post(&format!("/v1/plans/{plan_id}/validate"), &json!({}))
        .await?;
    println!("{}", serde_json::to_string_pretty(&body)?);
    Ok(())
}
