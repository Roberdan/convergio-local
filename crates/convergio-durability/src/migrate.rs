//! Migration runner.
//!
//! Applies the Layer 1 SQLite migrations.
//!
//! `set_ignore_missing(true)` lets us coexist with other crates'
//! migrators on the same `_sqlx_migrations` table — each crate owns its
//! own version range.

use crate::error::Result;
use convergio_db::Pool;

/// Apply pending migrations against the supplied pool.
///
/// Idempotent — safe to call on every daemon start.
pub async fn init(pool: &Pool) -> Result<()> {
    let mut migrator = sqlx::migrate!("./migrations");
    migrator.set_ignore_missing(true);
    migrator.run(pool.inner()).await?;
    tracing::info!("durability migrations up to date");
    Ok(())
}
