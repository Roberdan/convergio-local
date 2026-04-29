//! Gate pipeline.
//!
//! A gate is a precondition that must hold before a state transition is
//! persisted. Gates run server-side in a fixed order (see
//! [`crate::Durability::transition_task`]):
//!
//! ```text
//! plan_status → evidence → crdt_conflict → no_debt → no_stub
//! → no_secrets → zero_warnings → wave_sequence
//! ```
//!
//! Adding a gate:
//!
//! 1. Implement the [`Gate`] trait in a new file under `gates/`.
//! 2. Register it in [`default_pipeline`].
//! 3. Document the rationale in an ADR.

mod crdt_conflict_gate;
mod evidence_gate;
mod no_debt_gate;
mod no_secrets_gate;
mod no_stub_gate;
mod plan_status_gate;
mod wave_sequence_gate;
mod zero_warnings_gate;

pub use crdt_conflict_gate::CrdtConflictGate;
pub use evidence_gate::EvidenceGate;
pub use no_debt_gate::{DebtRule, NoDebtGate};
pub use no_secrets_gate::{NoSecretsGate, SecretRule};
pub use no_stub_gate::{NoStubGate, StubRule};
pub use plan_status_gate::PlanStatusGate;
pub use wave_sequence_gate::WaveSequenceGate;
pub use zero_warnings_gate::ZeroWarningsGate;

use crate::error::Result;
use crate::model::{Task, TaskStatus};
use convergio_db::Pool;
use std::sync::Arc;

/// Context handed to every gate.
#[derive(Clone)]
pub struct GateContext {
    /// DB pool.
    pub pool: Pool,
    /// Task before the proposed transition.
    pub task: Task,
    /// Status the caller wants to move to.
    pub target_status: TaskStatus,
    /// Agent claiming the transition (if any).
    pub agent_id: Option<String>,
}

/// One gate.
#[async_trait::async_trait]
pub trait Gate: Send + Sync {
    /// Stable name (used in error messages and ADRs).
    fn name(&self) -> &'static str;
    /// Returns `Ok(())` to allow, `Err(GateRefused { ... })` to block.
    async fn check(&self, ctx: &GateContext) -> Result<()>;
}

/// Erased pipeline.
pub type Pipeline = Vec<Arc<dyn Gate>>;

/// Default pipeline. Order is meaningful — see module docs.
///
/// Order rationale:
/// 1. `PlanStatusGate` first (cheap, refuses if the plan is dead).
/// 2. `EvidenceGate` second (refuses if required kinds missing).
/// 3. `CrdtConflictGate` — unresolved metadata conflicts block completion.
/// 4. `NoDebtGate` (P1) — debt markers in payloads.
/// 5. `NoStubGate` (P4) — scaffolding markers in payloads.
/// 6. `NoSecretsGate` (P2) — common credential leaks in payloads.
/// 7. `ZeroWarningsGate` (P1) — build/lint/test signal must be clean.
/// 8. `WaveSequenceGate` last (queries dependencies in the same plan).
pub fn default_pipeline() -> Pipeline {
    vec![
        Arc::new(PlanStatusGate),
        Arc::new(EvidenceGate),
        Arc::new(CrdtConflictGate),
        Arc::new(NoDebtGate::default()),
        Arc::new(NoStubGate::default()),
        Arc::new(NoSecretsGate::default()),
        Arc::new(ZeroWarningsGate),
        Arc::new(WaveSequenceGate),
    ]
}

/// Run every gate in `pipeline` against `ctx`, short-circuiting on the
/// first refusal.
pub async fn run(pipeline: &Pipeline, ctx: &GateContext) -> Result<()> {
    for gate in pipeline {
        gate.check(ctx).await?;
    }
    Ok(())
}
