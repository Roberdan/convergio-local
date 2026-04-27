//! # convergio-lifecycle — Layer 3 (skeleton)
//!
//! Spawn, supervise, heartbeat and reap long-running agent processes.
//!
//! Layer 3 owns the `agent_processes` table and the reaper loop. It
//! launches arbitrary executables (Claude Code session, Python script,
//! shell command), tracks PID + exit code, and notifies Layer 1 when an
//! agent dies so the task it was holding can be re-queued.
//!
//! ## Status
//!
//! Crate skeleton — see [ROADMAP.md](../../../ROADMAP.md) week 3-4.
//!
//! ## What it is NOT
//!
//! - **Not** systemd or launchd — we don't manage system services.
//! - **Not** a sandbox — agents run with the daemon's privileges.
//! - **Not** Kubernetes — no resource limits, no scheduling, no networking.

#![forbid(unsafe_code)]
#![allow(missing_docs)]

/// Placeholder for the Layer 3 facade.
pub struct Supervisor;

impl Supervisor {
    /// Build a supervisor over the given pool.
    pub fn new(_pool: convergio_db::Pool) -> Self {
        Self
    }
}
