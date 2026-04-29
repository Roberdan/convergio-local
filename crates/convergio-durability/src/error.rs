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

    /// A cell contains operations that declare different CRDT types.
    #[error("mixed crdt types for {entity_type}/{entity_id}/{field_name}")]
    MixedCrdtTypes {
        /// Logical entity type.
        entity_type: String,
        /// Logical entity id.
        entity_id: String,
        /// Field name within the entity.
        field_name: String,
    },

    /// A CRDT operation type is not supported by the merge engine.
    #[error("unsupported crdt type: {crdt_type}")]
    UnsupportedCrdtType {
        /// Unsupported CRDT type.
        crdt_type: String,
    },

    /// A CRDT operation kind is not supported for the declared type.
    #[error("unsupported crdt operation: {crdt_type}/{op_kind}")]
    UnsupportedCrdtOperation {
        /// Declared CRDT type.
        crdt_type: String,
        /// Unsupported operation kind.
        op_kind: String,
    },

    /// A CRDT operation payload does not match the declared type.
    #[error("invalid crdt value for {crdt_type}: {reason}")]
    InvalidCrdtValue {
        /// Declared CRDT type.
        crdt_type: String,
        /// Validation failure reason.
        reason: String,
    },

    /// A workspace resource is already leased by another active agent.
    #[error("workspace lease conflict: resource={resource_id} lease={lease_id} agent={agent_id}")]
    WorkspaceLeaseConflict {
        /// Leased resource id.
        resource_id: String,
        /// Active conflicting lease id.
        lease_id: String,
        /// Agent holding the active lease.
        agent_id: String,
    },

    /// Workspace lease request is invalid.
    #[error("invalid workspace lease: {reason}")]
    InvalidWorkspaceLease {
        /// Validation failure reason.
        reason: String,
    },

    /// A patch proposal violates workspace coordination policy.
    #[error("workspace patch refused: {kind}: {reason}")]
    WorkspacePatchRefused {
        /// Conflict kind.
        kind: String,
        /// Human-readable refusal reason.
        reason: String,
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
