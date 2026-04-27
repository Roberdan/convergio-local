//! # convergio-thor — Layer 4 (skeleton)
//!
//! Validator agent. Reads completed tasks + their evidence, produces a
//! verdict before the plan is closed. Runs as a separate Layer 3
//! process so it cannot be confused with the executor.
//!
//! ## Status
//!
//! Crate skeleton — see [ROADMAP.md](../../../ROADMAP.md) week 5-6.

#![forbid(unsafe_code)]
#![allow(missing_docs)]

/// Validator verdict.
pub enum Verdict {
    /// Plan passes — safe to close.
    Pass,
    /// Plan fails — list of reasons.
    Fail(Vec<String>),
}

/// Placeholder for the validator.
pub struct Thor;
