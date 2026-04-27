//! Convergio daemon entry point.
//!
//! Boots the HTTP server, runs Layer 1 + Layer 2 migrations, and spawns
//! the reaper loop. Mode is a function of `CONVERGIO_DB`:
//!
//! - `sqlite://...` (or unset → `~/.convergio/state.db`) — personal
//! - `postgres://...` — team (deferred)

use chrono::Duration;
use convergio_bus::Bus;
use convergio_db::Pool;
use convergio_durability::reaper::{self, ReaperConfig};
use convergio_durability::{init as init_durability, Durability};
use convergio_lifecycle::Supervisor;
use convergio_server::{router, AppState};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_env("CONVERGIO_LOG").unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let db_url = std::env::var("CONVERGIO_DB").unwrap_or_else(|_| default_sqlite_url());
    let bind = std::env::var("CONVERGIO_BIND")
        .unwrap_or_else(|_| "127.0.0.1:8420".into())
        .parse::<SocketAddr>()?;

    tracing::info!(%db_url, %bind, "starting convergio daemon");

    let pool = Pool::connect(&db_url).await?;
    init_durability(&pool).await?;
    convergio_bus::init(&pool).await?;
    convergio_lifecycle::init(&pool).await?;

    let durability = Arc::new(Durability::new(pool.clone()));
    let bus = Arc::new(Bus::new(pool.clone()));
    let supervisor = Arc::new(Supervisor::new(pool));

    let reaper_config = ReaperConfig {
        timeout: Duration::seconds(parse_env_i64("CONVERGIO_REAPER_TIMEOUT_SECS", 300)),
        tick_interval: Duration::seconds(parse_env_i64("CONVERGIO_REAPER_TICK_SECS", 60)),
    };
    let _reaper = reaper::spawn(durability.clone(), reaper_config);

    let state = AppState {
        durability: durability.clone(),
        bus: bus.clone(),
        supervisor: supervisor.clone(),
    };
    let app = router(state);

    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!(%bind, "listening");
    axum::serve(listener, app).await?;
    Ok(())
}

fn default_sqlite_url() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    format!("sqlite://{home}/.convergio/state.db?mode=rwc")
}

fn parse_env_i64(key: &str, default: i64) -> i64 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(default)
}
