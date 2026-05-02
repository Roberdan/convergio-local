//! Step driver for `cvg update` (F50). Sequence: probe -> rebuild
//! three crates -> sync `~/.cargo/bin` to `~/.local/bin` (F44) ->
//! pkill + restart daemon with `~/.cargo/bin` first on PATH and
//! `CONVERGIO_EXPECTED_VERSION` set (F45) -> re-probe + verify audit
//! chain. `--if-needed` short-circuits when versions already match.

use super::Client;
use super::OutputMode;
use anyhow::{Context, Result};
use convergio_i18n::Bundle;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// Caller-visible flags translated from clap.
#[derive(Clone, Copy, Debug)]
pub struct UpdateOptions {
    /// Skip the rebuild when daemon already matches workspace.
    pub if_needed: bool,
    /// Rebuild and sync, but do not restart the daemon.
    pub skip_restart: bool,
}

/// What `cvg update` produced. Rendered by the caller.
#[derive(Clone, Debug)]
pub struct UpdateOutcome {
    /// True when at least one binary was rebuilt.
    pub rebuilt: bool,
    /// True when the daemon was restarted.
    pub restarted: bool,
    /// Daemon version reported before the update (or "unknown").
    pub prior_version: String,
    /// Daemon version reported after the update (or workspace version
    /// when daemon is unreachable post-restart).
    pub new_version: String,
    /// Audit chain verification result post-restart.
    pub audit_chain_ok: bool,
    /// True when `--if-needed` short-circuited the rebuild.
    pub skipped_no_update_needed: bool,
}

/// Drive the update steps and return the outcome record.
pub async fn run_update(
    client: &Client,
    bundle: &Bundle,
    output: OutputMode,
    opts: UpdateOptions,
) -> Result<UpdateOutcome> {
    let workspace_version = env!("CARGO_PKG_VERSION").to_string();
    let prior_version = probe_daemon_version(client)
        .await
        .unwrap_or_else(|_| "unknown".into());

    if opts.if_needed && prior_version == workspace_version {
        return Ok(UpdateOutcome {
            rebuilt: false,
            restarted: false,
            prior_version: prior_version.clone(),
            new_version: prior_version,
            audit_chain_ok: probe_audit(client).await.unwrap_or(false),
            skipped_no_update_needed: true,
        });
    }

    if matches!(output, OutputMode::Human) {
        println!("{}", bundle.t("update-rebuild-header", &[]));
    }
    rebuild_all(bundle, output)?;

    if matches!(output, OutputMode::Human) {
        println!("{}", bundle.t("update-sync-header", &[]));
    }
    sync_shadowed_binaries(bundle)?;

    let restarted = if opts.skip_restart {
        if matches!(output, OutputMode::Human) {
            println!("{}", bundle.t("update-restart-skipped", &[]));
        }
        false
    } else {
        if matches!(output, OutputMode::Human) {
            println!("{}", bundle.t("update-restart-header", &[]));
        }
        restart_daemon(&workspace_version)?;
        true
    };

    if matches!(output, OutputMode::Human) {
        println!("{}", bundle.t("update-verify-header", &[]));
    }
    let new_version = probe_daemon_version(client)
        .await
        .unwrap_or_else(|_| workspace_version.clone());
    let audit_chain_ok = probe_audit(client).await.unwrap_or(false);

    Ok(UpdateOutcome {
        rebuilt: true,
        restarted,
        prior_version,
        new_version,
        audit_chain_ok,
        skipped_no_update_needed: false,
    })
}

async fn probe_daemon_version(client: &Client) -> Result<String> {
    let body: Value = client.get("/v1/health").await?;
    Ok(body
        .get("running_version")
        .and_then(Value::as_str)
        .or_else(|| body.get("version").and_then(Value::as_str))
        .unwrap_or("unknown")
        .to_string())
}

async fn probe_audit(client: &Client) -> Result<bool> {
    let body: Value = client.get("/v1/audit/verify").await?;
    Ok(body.get("ok").and_then(Value::as_bool).unwrap_or(false))
}

fn cargo_bin() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".cargo").join("bin"))
}

fn local_bin() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".local").join("bin"))
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn rebuild_all(bundle: &Bundle, output: OutputMode) -> Result<()> {
    let workspace_root = workspace_root().context("locate workspace root")?;
    for crate_name in ["convergio-server", "convergio-cli", "convergio-mcp"] {
        if matches!(output, OutputMode::Human) {
            println!(
                "  {}",
                bundle.t("update-rebuild-step", &[("crate", crate_name)])
            );
        }
        run_step(
            "cargo install",
            Command::new("cargo")
                .arg("install")
                .arg("--path")
                .arg(workspace_root.join("crates").join(crate_name))
                .arg("--force")
                .arg("--locked"),
        )?;
    }
    Ok(())
}

fn sync_shadowed_binaries(bundle: &Bundle) -> Result<()> {
    let cargo_bin = cargo_bin().context("HOME is not set")?;
    let local_bin = local_bin().context("HOME is not set")?;
    if !local_bin.is_dir() {
        // F44: ~/.local/bin may not exist on a fresh box. Don't error,
        // just skip — `cvg doctor` already covers binary discovery.
        return Ok(());
    }
    for bin in ["convergio", "cvg", "convergio-mcp"] {
        let src = cargo_bin.join(bin);
        let dst = local_bin.join(bin);
        if src.is_file() {
            // F44 contract: always overwrite, regardless of which copy
            // PATH currently resolves to.
            if let Err(e) = std::fs::copy(&src, &dst) {
                let src = src.display().to_string();
                let dst = dst.display().to_string();
                let reason = e.to_string();
                eprintln!(
                    "{}",
                    bundle.t(
                        "update-sync-copy-warning",
                        &[("src", &src), ("dst", &dst), ("reason", &reason)]
                    )
                );
            }
        }
    }
    Ok(())
}

fn restart_daemon(workspace_version: &str) -> Result<()> {
    // Best-effort kill of any running daemon. `pkill` returning 1 is
    // fine (no match); other failures are real.
    let kill_status = Command::new("pkill")
        .args(["-f", "convergio start"])
        .status();
    if let Ok(s) = kill_status {
        if !s.success() && s.code() != Some(1) {
            anyhow::bail!("pkill convergio failed with status {s}");
        }
    }
    std::thread::sleep(Duration::from_millis(800));

    // F45: ensure ~/.cargo/bin is first on PATH so cargo metadata
    // (used by `cvg graph build`) resolves under launchd-spawned
    // daemons too.
    let cargo_bin = cargo_bin().context("HOME is not set")?;
    let new_path = match std::env::var("PATH") {
        Ok(p) => format!("{}:{}", cargo_bin.display(), p),
        Err(_) => format!("{}", cargo_bin.display()),
    };

    let log_path = home_dir()
        .map(|h| h.join(".convergio").join("daemon.log"))
        .unwrap_or_else(|| PathBuf::from("/tmp/convergio-daemon.log"));
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let log_handle = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("open daemon log {}", log_path.display()))?;
    let log_err = log_handle.try_clone().context("clone daemon log handle")?;

    let convergio_bin = which_or_default(&cargo_bin, "convergio");
    Command::new(convergio_bin)
        .arg("start")
        .env("PATH", &new_path)
        .env("CONVERGIO_EXPECTED_VERSION", workspace_version)
        .stdout(log_handle)
        .stderr(log_err)
        .spawn()
        .context("spawn convergio start")?;
    std::thread::sleep(Duration::from_secs(3));
    Ok(())
}

fn which_or_default(cargo_bin: &Path, name: &str) -> PathBuf {
    let candidate = cargo_bin.join(name);
    if candidate.is_file() {
        candidate
    } else {
        PathBuf::from(name)
    }
}

fn workspace_root() -> Result<PathBuf> {
    // Walk up from CWD looking for the top-level Cargo.toml that
    // declares `[workspace]`. Avoids a hard dependency on cargo
    // metadata at runtime.
    let mut here = std::env::current_dir().context("cwd")?;
    loop {
        let candidate = here.join("Cargo.toml");
        if candidate.is_file() {
            if let Ok(text) = std::fs::read_to_string(&candidate) {
                if text.contains("[workspace]") {
                    return Ok(here);
                }
            }
        }
        if !here.pop() {
            anyhow::bail!("could not find workspace Cargo.toml above CWD");
        }
    }
}

fn run_step(label: &str, cmd: &mut Command) -> Result<()> {
    let status = cmd.status().with_context(|| format!("spawn {label}"))?;
    if !status.success() {
        anyhow::bail!("{label} failed with status {}", status.code().unwrap_or(-1));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_root_finds_repo_root() {
        let root = workspace_root().expect("find workspace root");
        let toml = std::fs::read_to_string(root.join("Cargo.toml")).expect("read root toml");
        assert!(toml.contains("[workspace]"));
    }

    #[test]
    fn which_or_default_prefers_cargo_bin_when_present() {
        let tempdir = tempfile::tempdir().expect("temp");
        let bin = tempdir.path().join("cvg");
        std::fs::write(&bin, "#!/bin/sh\n").expect("write");
        assert_eq!(which_or_default(tempdir.path(), "cvg"), bin);
    }

    #[test]
    fn which_or_default_falls_back_to_name() {
        let tempdir = tempfile::tempdir().expect("temp");
        assert_eq!(
            which_or_default(tempdir.path(), "missing-bin"),
            PathBuf::from("missing-bin")
        );
    }
}
