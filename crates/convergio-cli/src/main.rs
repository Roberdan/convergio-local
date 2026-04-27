//! `cvg` — Convergio command-line interface.
//!
//! Pure HTTP client for the Convergio daemon. Does **not** import any
//! server crate — the daemon is the source of truth.

use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "cvg", version, about = "Convergio CLI", long_about = None)]
struct Cli {
    /// Daemon base URL.
    #[arg(long, env = "CONVERGIO_URL", default_value = "http://127.0.0.1:8420")]
    url: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Probe the daemon.
    Health,
    /// Plan operations.
    Plan {
        #[command(subcommand)]
        sub: commands::plan::PlanCommand,
    },
    /// Audit log operations.
    Audit {
        #[command(subcommand)]
        sub: commands::audit::AuditCommand,
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = commands::Client::new(cli.url);
    match cli.command {
        Command::Health => commands::health::run(&client).await,
        Command::Plan { sub } => commands::plan::run(&client, sub).await,
        Command::Audit { sub } => commands::audit::run(&client, sub).await,
        Command::Solve { mission } => commands::solve::run(&client, &mission).await,
        Command::Dispatch => commands::dispatch::run(&client).await,
        Command::Validate { plan_id } => commands::validate::run(&client, &plan_id).await,
    }
}
