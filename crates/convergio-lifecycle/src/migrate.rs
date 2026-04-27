//! Migration runner for Layer 3.
//!
//! Coexists with Layer 1/2 migrators on the shared `_sqlx_migrations`
//! table by toggling `set_ignore_missing(true)`. Lifecycle owns version
//! 201+.

use crate::error::Result;
use convergio_db::Pool;

/// Apply pending migrations against the supplied pool.
///
/// Idempotent — safe to call on every daemon start.
pub async fn init(pool: &Pool) -> Result<()> {
    let mut migrator = sqlx::migrate!("./migrations");
    migrator.set_ignore_missing(true);
    migrator.run(pool.inner()).await?;
    tracing::info!("lifecycle migrations up to date");
    Ok(())
}
