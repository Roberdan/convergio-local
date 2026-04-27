//! Migration runner.
//!
//! Schema is identical between SQLite and Postgres for the MVP. When a
//! backend-specific migration is needed (e.g. JSONB columns), introduce a
//! second migration directory and dispatch on
//! [`convergio_db::Backend`].

use crate::error::Result;
use convergio_db::Pool;
use sqlx::migrate::Migrator;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Apply pending migrations against the supplied pool.
///
/// Idempotent — safe to call on every daemon start.
pub async fn init(pool: &Pool) -> Result<()> {
    MIGRATOR.run(pool.inner()).await?;
    tracing::info!("migrations up to date");
    Ok(())
}
