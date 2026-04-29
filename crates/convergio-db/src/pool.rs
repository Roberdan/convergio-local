//! Pool wrapper.
//!
//! Convergio is local-first and SQLite-only. `Pool` is a thin newtype
//! around [`sqlx::SqlitePool`] so callers don't depend on `sqlx`
//! directly.

use crate::error::{DbError, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::Path;
use std::str::FromStr;
use tracing::info;

/// The database backend currently in use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// SQLite — local mode.
    Sqlite,
}

/// SQLite connection pool.
///
/// Created once at daemon startup, cloned (cheaply) into every request
/// extractor and background loop.
#[derive(Clone)]
pub struct Pool {
    inner: SqlitePool,
    backend: Backend,
}

impl Pool {
    /// Connect to the database identified by `url`.
    ///
    /// Only `sqlite://` URLs are accepted.
    pub async fn connect(url: &str) -> Result<Self> {
        let backend = detect_backend(url)?;
        ensure_sqlite_parent(url)?;
        let opts = SqliteConnectOptions::from_str(url)
            .map_err(|e| DbError::InvalidUrl(e.to_string()))?
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(16)
            .connect_with(opts)
            .await?;

        info!(?backend, "connected to database");
        Ok(Self {
            inner: pool,
            backend,
        })
    }

    /// Backend in use.
    pub fn backend(&self) -> Backend {
        self.backend
    }

    /// Borrow the underlying [`sqlx::SqlitePool`].
    pub fn inner(&self) -> &SqlitePool {
        &self.inner
    }
}

fn detect_backend(url: &str) -> Result<Backend> {
    let scheme = url
        .split_once("://")
        .map(|(s, _)| s)
        .ok_or_else(|| DbError::InvalidUrl(format!("missing scheme in {url}")))?;
    match scheme {
        "sqlite" => Ok(Backend::Sqlite),
        other => Err(DbError::UnsupportedScheme(other.into())),
    }
}

fn ensure_sqlite_parent(url: &str) -> Result<()> {
    let trimmed = url.trim_start_matches("sqlite://");
    if trimmed.starts_with(":memory:") || trimmed.contains("mode=memory") {
        return Ok(());
    }
    let without_query = trimmed.split_once('?').map(|(p, _)| p).unwrap_or(trimmed);
    let path = Path::new(without_query);
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_sqlite_scheme() {
        assert_eq!(detect_backend("sqlite://./x.db").unwrap(), Backend::Sqlite);
    }

    #[test]
    fn rejects_unknown_scheme() {
        assert!(detect_backend("mysql://x").is_err());
        assert!(detect_backend("file://x").is_err());
        assert!(detect_backend("not-a-url").is_err());
    }

    #[tokio::test]
    async fn connect_to_sqlite_in_tempdir() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/dirs/state.db");
        let url = format!("sqlite://{}", path.display());
        let pool = Pool::connect(&url).await.unwrap();
        assert_eq!(pool.backend(), Backend::Sqlite);
        assert!(path.exists());
    }
}
