//! `cvg service ...` — install and control the user daemon service.

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use convergio_i18n::Bundle;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const LABEL: &str = "com.convergio.v3";
const SERVICE: &str = "convergio.service";

/// User-level service subcommands.
#[derive(Subcommand)]
pub enum ServiceCommand {
    /// Write the user service file.
    Install {
        /// Overwrite an existing service file.
        #[arg(long)]
        force: bool,
    },
    /// Start or reload the user service.
    Start,
    /// Stop the user service.
    Stop,
    /// Show whether the service manager reports it as loaded.
    Status,
    /// Stop and remove the user service file.
    Uninstall,
}

/// Run a service subcommand.
pub async fn run(bundle: &Bundle, cmd: ServiceCommand) -> Result<()> {
    let service = ServiceSpec::current()?;
    match cmd {
        ServiceCommand::Install { force } => {
            service.install(force)?;
            println!(
                "{}",
                bundle.t(
                    "service-installed",
                    &[("path", &service.path.display().to_string())]
                )
            );
        }
        ServiceCommand::Start => {
            service.start()?;
            println!("{}", bundle.t("service-started", &[]));
        }
        ServiceCommand::Stop => {
            service.stop()?;
            println!("{}", bundle.t("service-stopped", &[]));
        }
        ServiceCommand::Status => {
            let key = if service.is_loaded()? {
                "service-status-loaded"
            } else {
                "service-status-not-loaded"
            };
            println!("{}", bundle.t(key, &[]));
        }
        ServiceCommand::Uninstall => {
            service.stop_best_effort();
            if service.path.exists() {
                fs::remove_file(&service.path)
                    .with_context(|| format!("remove {}", service.path.display()))?;
            }
            println!("{}", bundle.t("service-uninstalled", &[]));
        }
    }
    Ok(())
}

enum ServiceKind {
    Launchd,
    Systemd,
}

struct ServiceSpec {
    kind: ServiceKind,
    path: PathBuf,
    content: String,
}

impl ServiceSpec {
    fn current() -> Result<Self> {
        let home = home()?;
        let convergio = resolve_binary("convergio")?;
        if cfg!(target_os = "macos") {
            let path = home
                .join("Library/LaunchAgents")
                .join(format!("{LABEL}.plist"));
            Ok(Self {
                kind: ServiceKind::Launchd,
                path,
                content: launchd_plist(&convergio, &home),
            })
        } else if cfg!(target_os = "linux") {
            let path = home.join(".config/systemd/user").join(SERVICE);
            Ok(Self {
                kind: ServiceKind::Systemd,
                path,
                content: systemd_unit(&convergio),
            })
        } else {
            bail!("user service management is supported on macOS and Linux")
        }
    }

    fn install(&self, force: bool) -> Result<()> {
        if self.path.exists() && !force {
            return Ok(());
        }
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        fs::write(&self.path, &self.content)
            .with_context(|| format!("write {}", self.path.display()))
    }

    fn start(&self) -> Result<()> {
        self.install(false)?;
        match self.kind {
            ServiceKind::Launchd => run_cmd(
                "launchctl",
                &[
                    "bootstrap",
                    &format!("gui/{}", uid()?),
                    path_str(&self.path)?,
                ],
            ),
            ServiceKind::Systemd => {
                run_cmd("systemctl", &["--user", "daemon-reload"])?;
                run_cmd("systemctl", &["--user", "enable", "--now", SERVICE])
            }
        }
    }

    fn stop(&self) -> Result<()> {
        match self.kind {
            ServiceKind::Launchd => run_cmd(
                "launchctl",
                &["bootout", &format!("gui/{}", uid()?), path_str(&self.path)?],
            ),
            ServiceKind::Systemd => run_cmd("systemctl", &["--user", "stop", SERVICE]),
        }
    }

    fn stop_best_effort(&self) {
        let _ = self.stop();
    }

    fn is_loaded(&self) -> Result<bool> {
        let ok = match self.kind {
            ServiceKind::Launchd => Command::new("launchctl")
                .args(["print", &format!("gui/{}/{}", uid()?, LABEL)])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status(),
            ServiceKind::Systemd => Command::new("systemctl")
                .args(["--user", "is-active", "--quiet", SERVICE])
                .status(),
        };
        Ok(ok.map(|s| s.success()).unwrap_or(false))
    }
}

fn launchd_plist(convergio: &Path, home: &Path) -> String {
    let out = home.join(".convergio/convergio.log");
    let err = home.join(".convergio/convergio.err.log");
    let cargo_bin = home.join(".cargo/bin");
    // launchd starts processes with a minimal PATH (typically
    // /usr/bin:/bin) and an unstable cwd. Both bite `cvg graph build`,
    // which shells out to `cargo metadata`: that needs `cargo` on PATH
    // and a valid current_dir(). We extend PATH to include
    // `~/.cargo/bin` (the canonical install path) plus the usual
    // homebrew/system bins, and pin WorkingDirectory to $HOME so the
    // daemon always has a stable cwd. Closes friction-log F45.
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>Label</key><string>{LABEL}</string>
  <key>ProgramArguments</key><array><string>{}</string><string>start</string></array>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><true/>
  <key>WorkingDirectory</key><string>{}</string>
  <key>EnvironmentVariables</key><dict>
    <key>PATH</key><string>{}:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin</string>
    <key>HOME</key><string>{}</string>
  </dict>
  <key>StandardOutPath</key><string>{}</string>
  <key>StandardErrorPath</key><string>{}</string>
</dict></plist>
"#,
        convergio.display(),
        home.display(),
        cargo_bin.display(),
        home.display(),
        out.display(),
        err.display()
    )
}

fn systemd_unit(convergio: &Path) -> String {
    format!(
        "[Unit]\nDescription=Convergio local daemon\n\n[Service]\nExecStart={} start\nRestart=on-failure\n\n[Install]\nWantedBy=default.target\n",
        convergio.display()
    )
}

fn run_cmd(program: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(program).args(args).status();
    match status {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => bail!("{program} failed with {status}"),
        Err(e) => Err(e).with_context(|| format!("run {program}")),
    }
}

fn resolve_binary(name: &str) -> Result<PathBuf> {
    let paths = std::env::var_os("PATH").context("PATH is not set")?;
    for dir in std::env::split_paths(&paths) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    bail!("`{name}` not found in PATH; run scripts/install-local.sh")
}

fn path_str(path: &Path) -> Result<&str> {
    path.to_str()
        .with_context(|| format!("path is not valid UTF-8: {}", path.display()))
}

fn home() -> Result<PathBuf> {
    Ok(PathBuf::from(
        std::env::var("HOME").context("HOME is not set")?,
    ))
}

fn uid() -> Result<String> {
    let out = Command::new("id").arg("-u").output().context("run id -u")?;
    if !out.status.success() {
        bail!("id -u failed");
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn plist() -> String {
        launchd_plist(
            &PathBuf::from("/usr/local/bin/convergio"),
            &PathBuf::from("/Users/example"),
        )
    }

    #[test]
    fn launchd_plist_includes_cargo_bin_in_path_env() {
        let out = plist();
        assert!(
            out.contains("<key>PATH</key>"),
            "EnvironmentVariables.PATH missing — F45 fix would not apply"
        );
        assert!(
            out.contains("/Users/example/.cargo/bin"),
            "expected ~/.cargo/bin in PATH, got: {out}"
        );
    }

    #[test]
    fn launchd_plist_pins_working_directory_to_home() {
        let out = plist();
        assert!(out.contains("<key>WorkingDirectory</key><string>/Users/example</string>"));
    }

    #[test]
    fn launchd_plist_keeps_log_redirects() {
        let out = plist();
        assert!(out.contains("/Users/example/.convergio/convergio.log"));
        assert!(out.contains("/Users/example/.convergio/convergio.err.log"));
    }
}
