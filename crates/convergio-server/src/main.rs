//! Convergio daemon entry point.
//!
//! Boots the HTTP server and the (future) background loops. Mode is a
//! function of `CONVERGIO_DB`:
//!
//! - `sqlite://...` (or unset → `~/.convergio/state.db`) — personal
//! - `postgres://...` — team

use convergio_db::Pool;
use convergio_durability::{init, Durability};
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
    init(&pool).await?;

    let state = AppState {
        durability: Arc::new(Durability::new(pool)),
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
