//! Errors surfaced by the runner.

use thiserror::Error;

/// Result alias.
pub type Result<T, E = RunnerError> = std::result::Result<T, E>;

/// Things that can go wrong while preparing or executing a runner.
#[derive(Debug, Error)]
pub enum RunnerError {
    /// `parse_kind` got a string it does not recognise.
    #[error("invalid runner kind: {0} (expected `claude:<model>` or `copilot:<model>`)")]
    InvalidKind(String),

    /// The vendor CLI binary is missing from `PATH`.
    #[error("vendor CLI `{cli}` not on PATH; install it or add it to PATH")]
    CliMissing {
        /// Binary name (`claude` or `copilot`).
        cli: &'static str,
    },

    /// Subprocess execution failed before we could collect output.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Underlying durability error while loading task metadata.
    #[error(transparent)]
    Durability(#[from] convergio_durability::DurabilityError),

    /// Something else opaque.
    #[error("runner error: {0}")]
    Other(String),
}
