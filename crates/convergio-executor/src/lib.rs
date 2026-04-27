//! # convergio-executor — Layer 4 (skeleton)
//!
//! Dispatcher loop. Picks `pending` tasks whose wave is ready and asks
//! Layer 3 to spawn agents for them.
//!
//! ## Status
//!
//! Crate skeleton — see [ROADMAP.md](../../../ROADMAP.md) week 5-6.
//!
//! ## Design
//!
//! One executor loop, ticking every 30s, owns task dispatch. It is **not**
//! a workflow engine — it just translates Layer 1 state into Layer 3
//! spawn calls. If the loop dies, no state is lost (it lives in Layer 1).

#![forbid(unsafe_code)]
#![allow(missing_docs)]

/// Placeholder for the executor.
pub struct Executor;
