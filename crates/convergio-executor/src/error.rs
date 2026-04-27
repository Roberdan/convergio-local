//! Executor errors.

use thiserror::Error;

/// Errors the executor can produce.
#[derive(Debug, Error)]
pub enum ExecutorError {
    /// Layer 1 failure.
    #[error(transparent)]
    Durability(#[from] convergio_durability::DurabilityError),

    /// Layer 3 failure.
    #[error(transparent)]
    Lifecycle(#[from] convergio_lifecycle::LifecycleError),
}

/// Convenience alias.
pub type Result<T, E = ExecutorError> = std::result::Result<T, E>;
