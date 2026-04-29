//! # convergio-db
//!
//! SQLite database pool for the local Convergio runtime.
//!
//! Convergio is intentionally local-first: one daemon, one user, one
//! SQLite database file. Higher layers (`convergio-durability`,
//! `convergio-bus`, `convergio-lifecycle`) depend on this crate, never
//! on `sqlx` directly.
//!
//! ## Database URL
//!
//! [`Pool::connect`] accepts only `sqlite://` URLs. The server defaults
//! to `sqlite://$HOME/.convergio/state.db?mode=rwc`.
//!
//! ## Example
//!
//! ```no_run
//! use convergio_db::Pool;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let pool = Pool::connect("sqlite://./state.db").await?;
//! // pass `pool` to the higher-layer stores
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]

mod error;
mod pool;

pub use error::{DbError, Result};
pub use pool::{Backend, Pool};
