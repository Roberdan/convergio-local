//! Planner errors.

use thiserror::Error;

/// Planner errors.
#[derive(Debug, Error)]
pub enum PlannerError {
    /// Mission text was empty after trimming.
    #[error("mission is empty")]
    EmptyMission,

    /// Underlying durability failure.
    #[error(transparent)]
    Durability(#[from] convergio_durability::DurabilityError),
}

/// Convenience alias.
pub type Result<T, E = PlannerError> = std::result::Result<T, E>;
