//! # convergio-bus — Layer 2 (skeleton)
//!
//! Persistent agent-to-agent message bus, scoped to a single plan.
//! Topic-based publish/subscribe + direct messages with ack.
//! Persistent by default so a consumer crash does not lose messages.
//!
//! ## Status
//!
//! Crate skeleton only — see [ROADMAP.md](../../../ROADMAP.md) week 3-4.
//! Public surface is being designed; do not depend on it yet.
//!
//! ## Planned API
//!
//! ```ignore
//! let bus = Bus::new(pool);
//! bus.publish(plan_id, "task.done", &payload).await?;
//! let mut sub = bus.subscribe(plan_id, "task.done").await?;
//! while let Some(msg) = sub.next().await { /* ... */ }
//! ```

#![forbid(unsafe_code)]
#![allow(missing_docs)] // skeleton — relax docs lint until shipped

/// Placeholder for the Layer 2 facade.
///
/// Will own a `Pool`, run migrations for `agent_messages`, expose
/// `publish` / `subscribe` / `ack`.
pub struct Bus;

impl Bus {
    /// Build a bus over the given pool.
    pub fn new(_pool: convergio_db::Pool) -> Self {
        Self
    }
}
