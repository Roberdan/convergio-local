//! `cvg health` — probe the daemon.

use super::Client;
use anyhow::Result;
use convergio_i18n::Bundle;
use serde_json::Value;

/// Run the command.
pub async fn run(client: &Client, bundle: &Bundle) -> Result<()> {
    match client.get::<Value>("/v1/health").await {
        Ok(body) => {
            let version = body.get("version").and_then(Value::as_str).unwrap_or("?");
            println!("{}", bundle.t("health-ok", &[("version", version)]));
            Ok(())
        }
        Err(e) => {
            eprintln!(
                "{}",
                bundle.t(
                    "health-unreachable",
                    &[("url", client.base()), ("reason", &e.to_string())]
                )
            );
            Err(e)
        }
    }
}
