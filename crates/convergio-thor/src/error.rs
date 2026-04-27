//! Thor errors.

use thiserror::Error;

/// Errors the validator can produce.
#[derive(Debug, Error)]
pub enum ThorError {
    /// Plan id was not found.
    #[error("plan not found: {0}")]
    PlanNotFound(String),

    /// Underlying durability failure.
    #[error(transparent)]
    Durability(#[from] convergio_durability::DurabilityError),
}

/// Convenience alias.
pub type Result<T, E = ThorError> = std::result::Result<T, E>;
