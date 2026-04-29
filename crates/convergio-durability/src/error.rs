//! Errors for Layer 1.

use thiserror::Error;

/// All errors the durability layer can produce.
#[derive(Debug, Error)]
pub enum DurabilityError {
    /// A row that should exist does not.
    #[error("not found: {entity} id={id}")]
    NotFound {
        /// Logical entity name (`plan`, `task`, ...).
        entity: &'static str,
        /// Identifier the caller passed.
        id: String,
    },

    /// A gate refused the requested transition.
    #[error("gate refused: {gate}: {reason}")]
    GateRefused {
        /// Name of the refusing gate.
        gate: &'static str,
        /// Human-readable reason — included in API responses.
        reason: String,
    },

    /// Audit chain corruption detected.
    #[error("audit chain broken at seq={seq}")]
    AuditChainBroken {
        /// First sequence number where the chain breaks.
        seq: i64,
    },

    /// A CRDT operation reused an actor/counter pair with different data.
    #[error("crdt op conflict: actor={actor_id} counter={counter}")]
    CrdtOpConflict {
        /// Actor UUID.
        actor_id: String,
        /// Per-actor operation counter.
        counter: i64,
    },

    /// Underlying database error.
    #[error(transparent)]
    Db(#[from] convergio_db::DbError),

    /// Sqlx error not surfaced via `convergio-db`.
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    /// JSON serialization / deserialization error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// Migration runner failure.
    #[error(transparent)]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

/// Convenience alias.
pub type Result<T, E = DurabilityError> = std::result::Result<T, E>;
