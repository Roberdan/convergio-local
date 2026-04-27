//! # convergio-planner — Layer 4 (basic)
//!
//! Turns a natural-language mission into a structured plan stored via
//! Layer 1. **Reference implementation** — users can replace this
//! crate with their own without touching the daemon.
//!
//! ## MVP heuristic (deterministic, no LLM)
//!
//! - Each non-blank line of the mission becomes one task in wave 1.
//! - The plan title is the first line; the rest forms the description.
//! - No required evidence is set by default — the caller can edit the
//!   plan afterwards if they want stricter gates.
//!
//! This is intentionally dumb. Replacing it with an LLM-backed planner
//! is a *future* milestone — the current version exists so users can
//! adopt the durability layer without writing a planner first.
//!
//! ## Quickstart
//!
//! ```no_run
//! use convergio_db::Pool;
//! use convergio_durability::{init, Durability};
//! use convergio_planner::Planner;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let pool = Pool::connect("sqlite://./state.db").await?;
//! init(&pool).await?;
//! let dur = Durability::new(pool);
//! let planner = Planner::new(dur);
//! let plan_id = planner
//!     .solve("ship the mvp\nwrite docs\nopen the source")
//!     .await?;
//! # Ok(()) }
//! ```

#![forbid(unsafe_code)]

mod error;
mod planner;

pub use error::{PlannerError, Result};
pub use planner::Planner;
