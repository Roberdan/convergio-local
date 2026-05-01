//! # convergio-graph — Tier-3 code-graph layer
//!
//! Walks the workspace via [`syn`] + [`cargo_metadata`], persists a
//! node/edge graph in SQLite, and exposes queries that drive
//! context-pack delegation, drift detection, and cluster suggestions.
//!
//! See `docs/adr/0014-code-graph-tier3-retrieval.md` for the rationale.
//!
//! ## Layout
//!
//! - [`model`] — [`Node`], [`Edge`], [`NodeKind`], [`EdgeKind`].
//! - [`parse`] — file-level syn walker.
//! - [`meta`] — `cargo metadata` wrapper for crate-level edges.
//! - [`store`] — SQLite persistence (migration range 600-699).
//! - [`build`] — top-level orchestrator: meta + parse + store.
//!
//! ## Quickstart
//!
//! ```no_run
//! use convergio_db::Pool;
//! use convergio_graph::{build, Store};
//! use std::path::Path;
//!
//! # async fn run() -> anyhow::Result<()> {
//! let pool = Pool::connect("sqlite://./state.db?mode=rwc").await?;
//! let store = Store::new(pool);
//! let report = build(Path::new("."), &store, false).await?;
//! println!("nodes={} edges={}", report.nodes, report.edges);
//! # Ok(()) }
//! ```

#![forbid(unsafe_code)]

pub mod build;
pub mod error;
pub mod meta;
pub mod model;
pub mod parse;
pub mod store;

pub use build::build;
pub use error::{GraphError, Result};
pub use model::{BuildReport, Edge, EdgeKind, Node, NodeKind, DOCS_CRATE};
pub use store::Store;
