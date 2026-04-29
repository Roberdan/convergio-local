//! Data-access layer for plans, tasks, evidence, CRDT metadata and agents.
//!
//! Stores are thin: they handle the SQL and the (de)serialization, but
//! they don't enforce gates. Gates live in [`crate::gates`] and are run
//! by the [`crate::Durability`] facade before any state-changing call.

mod crdt;
mod evidence;
mod plans;
mod tasks;

pub use crdt::{AppendOutcome, CrdtActor, CrdtOp, CrdtStore, NewCrdtOp};
pub use evidence::EvidenceStore;
pub use plans::PlanStore;
pub use tasks::TaskStore;
