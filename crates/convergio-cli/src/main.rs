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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = commands::Client::new(cli.url);
    match cli.command {
        Command::Health => commands::health::run(&client).await,
        Command::Plan { sub } => commands::plan::run(&client, sub).await,
        Command::Audit { sub } => commands::audit::run(&client, sub).await,
    }
}
