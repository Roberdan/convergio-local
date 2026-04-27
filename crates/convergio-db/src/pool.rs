//! Pool wrapper that hides the SQLite/Postgres split.

use crate::error::{DbError, Result};
use sqlx::any::{install_default_drivers, AnyPoolOptions};
use sqlx::AnyPool;
use std::path::Path;
use tracing::info;

/// The database backend currently in use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// SQLite — personal mode.
    Sqlite,
    /// Postgres — team mode.
    Postgres,
}

/// A type-erased connection pool.
///
/// Created once at daemon startup, cloned (cheaply) into every request
/// extractor and background loop.
#[derive(Clone)]
pub struct Pool {
    inner: AnyPool,
    backend: Backend,
}

impl Pool {
    /// Connect to the database identified by `url`.
    ///
    /// SQLite parent directories are created on demand. Postgres URLs are
    /// passed through verbatim to sqlx.
    pub async fn connect(url: &str) -> Result<Self> {
        install_default_drivers();
        let backend = detect_backend(url)?;

        if matches!(backend, Backend::Sqlite) {
            ensure_sqlite_parent(url)?;
        }

        let pool = AnyPoolOptions::new()
            .max_connections(16)
            .connect(url)
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

    /// Borrow the underlying [`sqlx::AnyPool`].
    pub fn inner(&self) -> &AnyPool {
        &self.inner
    }
}

fn detect_backend(url: &str) -> Result<Backend> {
    let parsed = url::Url::parse(url).map_err(|e| DbError::InvalidUrl(e.to_string()))?;
    match parsed.scheme() {
        "sqlite" => {
            #[cfg(feature = "sqlite")]
            {
                Ok(Backend::Sqlite)
            }
            #[cfg(not(feature = "sqlite"))]
            {
                Err(DbError::UnsupportedScheme("sqlite".into()))
            }
        }
        "postgres" | "postgresql" => {
            #[cfg(feature = "postgres")]
            {
                Ok(Backend::Postgres)
            }
            #[cfg(not(feature = "postgres"))]
            {
                Err(DbError::UnsupportedScheme("postgres".into()))
            }
        }
        other => Err(DbError::UnsupportedScheme(other.into())),
    }
}

fn ensure_sqlite_parent(url: &str) -> Result<()> {
    // sqlite://./state.db   → ./state.db
    // sqlite:///abs/path.db → /abs/path.db
    // sqlite::memory:       → no parent
    let trimmed = url.trim_start_matches("sqlite://");
    if trimmed.starts_with(":memory:") || trimmed.contains("mode=memory") {
        return Ok(());
    }
    let path = Path::new(trimmed);
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
    fn detect_postgres_scheme_disabled_by_default() {
        // Default features include only sqlite.
        let err = detect_backend("postgres://user@host/db");
        assert!(err.is_err());
    }

    #[test]
    fn rejects_unknown_scheme() {
        assert!(detect_backend("mysql://x").is_err());
        assert!(detect_backend("not a url").is_err());
    }

    #[tokio::test]
    async fn connect_to_sqlite_in_tempdir() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/dirs/state.db");
        let url = format!("sqlite://{}?mode=rwc", path.display());
        let pool = Pool::connect(&url).await.unwrap();
        assert_eq!(pool.backend(), Backend::Sqlite);
        assert!(path.exists());
    }
}
