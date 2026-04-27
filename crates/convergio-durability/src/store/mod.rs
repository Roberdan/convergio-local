//! Data-access layer for plans, tasks, evidence and agents.
//!
//! Stores are thin: they handle the SQL and the (de)serialization, but
//! they don't enforce gates. Gates live in [`crate::gates`] and are run
//! by the [`crate::Durability`] facade before any state-changing call.

mod evidence;
mod plans;
mod tasks;

pub use evidence::EvidenceStore;
pub use plans::PlanStore;
pub use tasks::TaskStore;
