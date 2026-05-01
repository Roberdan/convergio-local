//! Error type for `convergio-graph`.

use thiserror::Error;

/// Top-level result alias.
pub type Result<T> = std::result::Result<T, GraphError>;

/// Errors emitted by the graph layer.
#[derive(Debug, Error)]
pub enum GraphError {
    /// Underlying SQLite / sqlx failure.
    #[error("database: {0}")]
    Db(#[from] convergio_db::DbError),

    /// Failure to invoke or parse `cargo metadata`.
    #[error("cargo metadata: {0}")]
    Metadata(#[from] cargo_metadata::Error),

    /// I/O failure while reading source files.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    /// `syn` parser error.
    #[error("syn parse {file}: {err}")]
    Syn {
        /// File that failed to parse.
        file: String,
        /// Underlying syn error.
        err: syn::Error,
    },

    /// `sqlx` runtime error.
    #[error("sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),

    /// Migration error.
    #[error("migrate: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    /// Generic.
    #[error("{0}")]
    Other(String),
}
