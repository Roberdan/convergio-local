//! Errors for Layer 2.

use thiserror::Error;

/// All errors the bus can produce.
#[derive(Debug, Error)]
pub enum BusError {
    /// Message id not found on ack.
    #[error("message not found: {0}")]
    NotFound(String),

    /// Topic does not match the requested scope (system vs plan-scoped).
    /// See ADR-0025.
    #[error("invalid topic scope: {0}")]
    InvalidTopicScope(String),

    /// Underlying database error.
    #[error(transparent)]
    Db(#[from] convergio_db::DbError),

    /// Sqlx error not surfaced via `convergio-db`.
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    /// Migration failure.
    #[error(transparent)]
    Migrate(#[from] sqlx::migrate::MigrateError),

    /// JSON serialization error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// Convenience alias.
pub type Result<T, E = BusError> = std::result::Result<T, E>;
