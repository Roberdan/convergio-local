//! `cvg update` — rebuild and restart the local Convergio daemon.
//!
//! Closes friction-log F50 for the dev-loop case: after a `git pull` or
//! `git merge` the locally installed `convergio`, `cvg`, and
//! `convergio-mcp` binaries no longer match the workspace tree. This
//! command rebuilds all three, syncs shadowed copies under
//! `~/.local/bin` (closes the in-scope half of F44), and restarts the
//! daemon with `~/.cargo/bin` first on PATH (closes the in-scope half
//! of F45).
//!
//! It is **not** an automated cron — wiring is in
//! [`lefthook.yml`](../../../../../lefthook.yml) under `post-merge`.
//! `--if-needed` makes the hook a no-op when running and expected
//! versions already match.
//!
//! See `update_run.rs` for the per-step driver.

use super::Client;
use super::OutputMode;
use anyhow::Result;
use convergio_i18n::Bundle;

use crate::commands::update_run::{run_update, UpdateOptions, UpdateOutcome};

/// Render mode chosen by the global `--output` flag.
fn render_outcome(bundle: &Bundle, outcome: &UpdateOutcome, output: OutputMode) -> Result<()> {
    match output {
        OutputMode::Human => {
            if outcome.skipped_no_update_needed {
                println!(
                    "{}",
                    bundle.t(
                        "update-no-update-needed",
                        &[("version", &outcome.new_version)]
                    )
                );
            } else {
                println!(
                    "{}",
                    bundle.t(
                        "update-summary-ok",
                        &[
                            ("prior", &outcome.prior_version),
                            ("new", &outcome.new_version),
                            ("restarted", if outcome.restarted { "yes" } else { "no" }),
                        ]
                    )
                );
            }
        }
        OutputMode::Json => {
            let payload = serde_json::json!({
                "rebuilt": outcome.rebuilt,
                "restarted": outcome.restarted,
                "prior_version": outcome.prior_version,
                "new_version": outcome.new_version,
                "audit_chain_ok": outcome.audit_chain_ok,
                "skipped_no_update_needed": outcome.skipped_no_update_needed,
            });
            println!("{}", serde_json::to_string_pretty(&payload)?);
        }
        OutputMode::Plain => {
            if outcome.skipped_no_update_needed {
                println!("noop {}", outcome.new_version);
            } else {
                println!(
                    "ok {} -> {} restarted={} audit={}",
                    outcome.prior_version,
                    outcome.new_version,
                    outcome.restarted,
                    outcome.audit_chain_ok
                );
            }
        }
    }
    Ok(())
}

/// Entry point wired from `main.rs`.
pub async fn run(
    client: &Client,
    bundle: &Bundle,
    output: OutputMode,
    if_needed: bool,
    skip_restart: bool,
) -> Result<()> {
    let opts = UpdateOptions {
        if_needed,
        skip_restart,
    };
    let outcome = run_update(client, bundle, output, opts).await?;
    render_outcome(bundle, &outcome, output)
}
