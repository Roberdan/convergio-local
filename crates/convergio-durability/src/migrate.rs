//! Migration runner.
//!
//! Schema is identical between SQLite and Postgres for the MVP. When a
//! backend-specific migration is needed (e.g. JSONB columns), introduce
//! a second migration directory and dispatch on
//! [`convergio_db::Backend`].
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
