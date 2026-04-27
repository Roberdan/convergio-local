//! Migration runner for Layer 2.
//!
//! `convergio-bus` and `convergio-durability` share the same database
//! and the same `_sqlx_migrations` tracking table, but each crate owns
//! its own migration files. We toggle `set_ignore_missing(true)` so
//! each migrator does not complain about rows it didn't write itself
//! (the durability migrator's version 1, the bus migrator's version
//! 101, and so on).

use crate::error::Result;
use convergio_db::Pool;

/// Apply pending migrations against the supplied pool.
///
/// Idempotent — safe to call on every daemon start.
pub async fn init(pool: &Pool) -> Result<()> {
    let mut migrator = sqlx::migrate!("./migrations");
    migrator.set_ignore_missing(true);
    migrator.run(pool.inner()).await?;
    tracing::info!("bus migrations up to date");
    Ok(())
}
