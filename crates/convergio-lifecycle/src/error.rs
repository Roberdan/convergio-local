//! Errors for Layer 3.

use thiserror::Error;

/// All errors the supervisor can produce.
#[derive(Debug, Error)]
pub enum LifecycleError {
    /// Process row not found.
    #[error("process not found: {0}")]
    NotFound(String),

    /// `tokio::process::Command::spawn` failed.
    #[error("spawn failed: {0}")]
    SpawnFailed(String),

    /// I/O error during process management.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Underlying database error.
    #[error(transparent)]
    Db(#[from] convergio_db::DbError),

    /// Sqlx error.
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    /// Migration runner failure.
    #[error(transparent)]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

/// Convenience alias.
pub type Result<T, E = LifecycleError> = std::result::Result<T, E>;
