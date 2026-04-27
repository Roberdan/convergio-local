//! i18n errors.

use thiserror::Error;

/// All errors the i18n layer can produce.
#[derive(Debug, Error)]
pub enum I18nError {
    /// The supplied locale tag is not one we ship.
    #[error("unsupported locale: {0}")]
    UnsupportedLocale(String),

    /// The bundled `.ftl` source failed to parse. This is a build-time
    /// bug: the bundle is included via `include_str!`, so a parse
    /// failure means the source we shipped is malformed.
    #[error("bundle parse error: {0}")]
    Parse(String),
}

/// Convenience alias.
pub type Result<T, E = I18nError> = std::result::Result<T, E>;
