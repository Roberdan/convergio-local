//! `cvg health` — probe the daemon.

use super::Client;
use super::OutputMode;
use anyhow::Result;
use convergio_i18n::Bundle;
use serde_json::Value;

/// Run the command.
pub async fn run(client: &Client, bundle: &Bundle, output: OutputMode) -> Result<()> {
    match client.get::<Value>("/v1/health").await {
        Ok(body) => {
            let version = body.get("version").and_then(Value::as_str).unwrap_or("?");
            let drift = body.get("drift").and_then(Value::as_bool).unwrap_or(false);
            let expected = body
                .get("expected_version")
                .and_then(Value::as_str)
                .unwrap_or("");
            match output {
                OutputMode::Human => {
                    println!("{}", bundle.t("health-ok", &[("version", version)]));
                    if drift {
                        println!(
                            "{}",
                            bundle.t(
                                "health-drift",
                                &[("running", version), ("expected", expected)]
                            )
                        );
                    }
                }
                OutputMode::Json => println!("{}", serde_json::to_string_pretty(&body)?),
                OutputMode::Plain => {
                    if drift {
                        println!("{version} drift");
                    } else {
                        println!("{version}");
                    }
                }
            }
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
