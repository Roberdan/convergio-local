//! `cvg setup` — initialize local user configuration.

use anyhow::{Context, Result};
use clap::{Subcommand, ValueEnum};
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
    /// Generate adapter snippets for an agent host.
    Agent {
        /// Agent host to configure.
        host: AgentHost,
        /// Overwrite existing snippets for this host.
        #[arg(long)]
        force: bool,
    },
}

/// Supported agent hosts for generated snippets.
#[derive(Clone, Copy, ValueEnum)]
pub enum AgentHost {
    /// Claude Desktop / Claude Code compatible MCP config.
    Claude,
    /// GitHub Copilot local IDE integrations.
    CopilotLocal,
    /// GitHub Copilot cloud agent repository hint.
    CopilotCloud,
    /// Cursor.
    Cursor,
    /// Cline.
    Cline,
    /// Continue.
    Continue,
    /// Qwen or qwen-code shell-style agent.
    Qwen,
    /// Generic shell agent.
    Shell,
}

impl AgentHost {
    fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::CopilotLocal => "copilot-local",
            Self::CopilotCloud => "copilot-cloud",
            Self::Cursor => "cursor",
            Self::Cline => "cline",
            Self::Continue => "continue",
            Self::Qwen => "qwen",
            Self::Shell => "shell",
        }
    }
}

/// Run setup. With no subcommand, runs `init`.
pub async fn run(bundle: &Bundle, cmd: Option<SetupCommand>) -> Result<()> {
    let command = cmd.unwrap_or(SetupCommand::Init { force: false });
    match command {
        SetupCommand::Init { force } => init(bundle, force),
        SetupCommand::Agent { host, force } => agent(bundle, host, force),
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

fn agent(bundle: &Bundle, host: AgentHost, force: bool) -> Result<()> {
    let home = convergio_home()?;
    let dir = home.join("adapters").join(host.as_str());
    fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;

    write_snippet(&dir.join("mcp.json"), &mcp_snippet(host), force)?;
    write_snippet(&dir.join("prompt.txt"), prompt_snippet(), force)?;
    write_snippet(&dir.join("README.txt"), &readme_snippet(host), force)?;

    println!(
        "{}",
        bundle.t(
            "setup-agent-created",
            &[
                ("host", host.as_str()),
                ("path", &dir.display().to_string())
            ]
        )
    );
    println!("{}", bundle.t("setup-agent-copy", &[]));
    Ok(())
}

fn write_snippet(path: &std::path::Path, content: &str, force: bool) -> Result<()> {
    if path.exists() && !force {
        return Ok(());
    }
    fs::write(path, content).with_context(|| format!("write {}", path.display()))
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

fn mcp_snippet(host: AgentHost) -> String {
    let name = if matches!(host, AgentHost::CopilotCloud) {
        "convergio-local"
    } else {
        "convergio"
    };
    format!(
        "{{\n  \"mcpServers\": {{\n    \"{name}\": {{\n      \"type\": \"stdio\",\n      \"command\": \"convergio-mcp\",\n      \"args\": [\"--url\", \"{DEFAULT_URL}\"]\n    }}\n  }}\n}}\n"
    )
}

fn prompt_snippet() -> &'static str {
    "Use Convergio as the local source of truth. Call convergio.help once. Use convergio.act for task lifecycle and evidence. If a gate refuses work, fix the reason, attach new evidence, and retry submit_task. Do not tell the user work is done until validate_plan returns Pass — agents submit, the validator (Thor) is the only path to done (ADR-0011).\n"
}

fn readme_snippet(host: AgentHost) -> String {
    format!(
        "Convergio adapter: {host}\n\n1. Ensure `convergio start` is running.\n2. Add mcp.json to the host's MCP configuration.\n3. Add prompt.txt to the agent's custom instructions.\n4. Run `cvg doctor --json` if the agent cannot connect.\n",
        host = host.as_str()
    )
}

fn is_current_config(path: &std::path::Path) -> Result<bool> {
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    Ok(content.contains(CONFIG_MARKER) && content.contains("version = 1"))
}
