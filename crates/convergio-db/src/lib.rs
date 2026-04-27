//! # convergio-db
//!
//! Database abstraction for the Convergio durability layer.
//!
//! This crate exposes a single [`Pool`] that backs both the **personal**
//! mode (SQLite at `~/.convergio/state.db`) and the **team** mode
//! (Postgres). Higher layers (`convergio-durability`, `convergio-bus`,
//! `convergio-lifecycle`) depend on this crate, never on `sqlx` directly,
//! so we can keep schema diffs between backends in one place.
//!
//! ## Backends
//!
//! Selected by the URL scheme passed to [`Pool::connect`]:
//!
//! | URL scheme | Backend | Feature flag |
//! |------------|---------|--------------|
//! | `sqlite://` | SQLite | `sqlite` (default) |
//! | `postgres://` | Postgres | `postgres` |
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
