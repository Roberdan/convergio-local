//! `cvg` — Convergio command-line interface.
//!
//! Pure HTTP client for the Convergio daemon. Does **not** import any
//! server crate — the daemon is the source of truth.
//!
//! All user-facing strings flow through [`convergio_i18n::Bundle`].
//! Locale resolution: `--lang` flag → `CONVERGIO_LANG` env →
//! `LANG`/`LC_ALL` env → fallback `en` (P5).

use anyhow::Result;
use clap::{Parser, Subcommand};
use convergio_i18n::{detect_locale, Bundle};

mod commands;

#[derive(Parser)]
#[command(name = "cvg", version, about = "Convergio CLI", long_about = None)]
struct Cli {
    /// Daemon base URL.
    #[arg(
        long,
        global = true,
        env = "CONVERGIO_URL",
        default_value = "http://127.0.0.1:8420"
    )]
    url: String,

    /// User interface language. Falls back to CONVERGIO_LANG / LANG / en.
    #[arg(long, global = true, value_name = "LOCALE")]
    lang: Option<String>,

    /// Output format for commands that support multiple views.
    #[arg(long, global = true, value_enum, default_value_t = commands::OutputMode::Human)]
    output: commands::OutputMode,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Probe the daemon.
    Health,
    /// Initialize local configuration.
    Setup {
        #[command(subcommand)]
        sub: Option<commands::setup::SetupCommand>,
    },
    /// Diagnose local configuration and daemon health.
    Doctor {
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Plan operations.
    Plan {
        #[command(subcommand)]
        sub: commands::plan::PlanCommand,
    },
    /// Task operations.
    Task {
        #[command(subcommand)]
        sub: commands::task::TaskCommand,
    },
    /// Evidence operations.
    Evidence {
        #[command(subcommand)]
        sub: commands::evidence::EvidenceCommand,
    },
    /// Audit log operations.
    Audit {
        #[command(subcommand)]
        sub: commands::audit::AuditCommand,
    },
    /// MCP bridge diagnostics.
    Mcp {
        #[command(subcommand)]
        sub: commands::mcp::McpCommand,
    },
    /// User-level daemon service management.
    Service {
        #[command(subcommand)]
        sub: commands::service::ServiceCommand,
    },
    /// Solve a mission into a plan (Layer 4 planner).
    Solve {
        /// Mission text — newline-separated tasks.
        mission: String,
    },
    /// Run one executor tick (dispatches pending tasks).
    Dispatch,
    /// Run Thor on a plan.
    Validate {
        /// Plan id.
        plan_id: String,
    },
    /// Run a guided local demo.
    Demo,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let locale = detect_locale(cli.lang.as_deref());
    let bundle = Bundle::new(locale).expect("default bundles always load");
    let client = commands::Client::new(cli.url);
    match cli.command {
        Command::Health => commands::health::run(&client, &bundle, cli.output).await,
        Command::Setup { sub } => commands::setup::run(&bundle, sub).await,
        Command::Doctor { json } => commands::doctor::run(&client, &bundle, cli.output, json).await,
        Command::Plan { sub } => commands::plan::run(&client, &bundle, sub).await,
        Command::Task { sub } => commands::task::run(&client, sub).await,
        Command::Evidence { sub } => commands::evidence::run(&client, sub).await,
        Command::Audit { sub } => commands::audit::run(&client, sub).await,
        Command::Mcp { sub } => commands::mcp::run(&bundle, sub).await,
        Command::Service { sub } => commands::service::run(&bundle, sub).await,
        Command::Solve { mission } => commands::solve::run(&client, &mission).await,
        Command::Dispatch => commands::dispatch::run(&client).await,
        Command::Validate { plan_id } => commands::validate::run(&client, &plan_id).await,
        Command::Demo => commands::demo::run(&client).await,
    }
}
