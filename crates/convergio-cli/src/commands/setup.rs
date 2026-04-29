//! `cvg setup` — initialize local user configuration.

use anyhow::{Context, Result};
use clap::Subcommand;
use convergio_i18n::Bundle;
use std::fs;
use std::path::PathBuf;

const DEFAULT_URL: &str = "http://127.0.0.1:8420";
const CONFIG_MARKER: &str = "# Convergio v3 local configuration";

/// Setup subcommands.
#[derive(Subcommand)]
pub enum SetupCommand {
    /// Generate local configuration.
    Init {
        /// Overwrite an existing config file.
        #[arg(long)]
        force: bool,
    },
}

/// Run setup. With no subcommand, runs `init`.
pub async fn run(bundle: &Bundle, cmd: Option<SetupCommand>) -> Result<()> {
    let command = cmd.unwrap_or(SetupCommand::Init { force: false });
    match command {
        SetupCommand::Init { force } => init(bundle, force),
    }
}

fn init(bundle: &Bundle, force: bool) -> Result<()> {
    let home = convergio_home()?;
    let adapters = home.join("adapters");
    fs::create_dir_all(&adapters).with_context(|| format!("create {}", adapters.display()))?;

    let config = home.join("config.toml");
    if config.exists() && !force && is_current_config(&config)? {
        println!(
            "{}",
            bundle.t(
                "setup-config-exists",
                &[("path", &config.display().to_string())]
            )
        );
    } else {
        if config.exists() {
            let backup = home.join("config.toml.v2.bak");
            fs::copy(&config, &backup)
                .with_context(|| format!("backup {} to {}", config.display(), backup.display()))?;
            println!(
                "{}",
                bundle.t(
                    "setup-config-backed-up",
                    &[("path", &backup.display().to_string())]
                )
            );
        }
        fs::write(&config, default_config())
            .with_context(|| format!("write {}", config.display()))?;
        println!(
            "{}",
            bundle.t(
                "setup-config-created",
                &[("path", &config.display().to_string())]
            )
        );
    }

    println!(
        "{}",
        bundle.t("setup-complete", &[("path", &home.display().to_string())])
    );
    println!("{}", bundle.t("setup-next-start", &[]));
    println!("{}", bundle.t("setup-next-doctor", &[]));
    Ok(())
}

fn convergio_home() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME is not set")?;
    Ok(PathBuf::from(home).join(".convergio"))
}

fn default_config() -> String {
    format!(
        "{CONFIG_MARKER}\n\
         version = 1\n\
         url = \"{DEFAULT_URL}\"\n\
         db = \"sqlite://$HOME/.convergio/v3/state.db?mode=rwc\"\n\
         bind = \"127.0.0.1:8420\"\n"
    )
}

fn is_current_config(path: &std::path::Path) -> Result<bool> {
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    Ok(content.contains(CONFIG_MARKER) && content.contains("version = 1"))
}
