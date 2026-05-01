//! `cvg validate <plan_id> [--wave N]` — Thor verdict.
//!
//! Without `--wave`, validation is plan-strict: every task must be
//! `submitted` or `done`, otherwise the verdict fails. With
//! `--wave N` (T3.06), validation is restricted to the named wave —
//! tasks in other waves do not block the verdict. This lets
//! long-running backlog plans (v0.1.x, v0.2, v0.3) close the OODA
//! loop wave by wave.

use super::Client;
use anyhow::Result;
use serde_json::{json, Value};

/// Run the command.
pub async fn run(client: &Client, plan_id: &str, wave: Option<i64>) -> Result<()> {
    let path = match wave {
        Some(w) => format!("/v1/plans/{plan_id}/validate?wave={w}"),
        None => format!("/v1/plans/{plan_id}/validate"),
    };
    let body: Value = client.post(&path, &json!({})).await?;
    println!("{}", serde_json::to_string_pretty(&body)?);
    Ok(())
}
