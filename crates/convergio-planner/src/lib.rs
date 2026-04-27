//! # convergio-planner — Layer 4 (skeleton)
//!
//! Turns a natural-language mission into a structured plan stored via
//! Layer 1. Reference implementation — users can replace it with their
//! own client without touching the daemon.
//!
//! ## Status
//!
//! Crate skeleton — see [ROADMAP.md](../../../ROADMAP.md) week 5-6.
//!
//! ## Planned API
//!
//! ```ignore
//! let planner = Planner::new(durability);
//! let plan_id = planner.solve("build me a todo CLI").await?;
//! ```

#![forbid(unsafe_code)]
#![allow(missing_docs)]

/// Placeholder for the planner facade.
pub struct Planner;
