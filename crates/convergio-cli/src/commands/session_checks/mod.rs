//! Per-check implementations for `cvg session pre-stop`.
//!
//! Each module is one [`Check`] from
//! [`crate::commands::session_pre_stop::registry`]. They share a
//! conservative shape: cheap (sub-second), shell-only, no daemon
//! writes. Checks that need async HTTP calls — bus inbound/outbound,
//! plan↔PR drift, handshake — stay as stubs in `session_pre_stop.rs`
//! until the trait is widened to async; their plan tasks are linked
//! from the `NotImplemented` outcome so operators can find the
//! follow-up.

pub mod friction_missing;
pub mod worktree_no_pr;
