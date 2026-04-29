//! `Durability` — the public facade tying stores, gates and audit log
//! together so that callers (HTTP layer, CLI) only see one type.

use crate::audit::{AuditLog, EntityKind};
use crate::error::Result;
use crate::gates::{self, GateContext, Pipeline};
use crate::model::{NewPlan, NewTask, Plan, Task, TaskStatus};
use crate::store::{EvidenceStore, PlanStore, TaskStore};
use convergio_db::Pool;
use serde_json::json;

/// Top-level Layer 1 handle.
///
/// Cheap to clone (clones the underlying pool). Hold one in your
/// application state and pass references into HTTP handlers.
#[derive(Clone)]
pub struct Durability {
    pool: Pool,
    pipeline: Pipeline,
}

impl Durability {
    /// Build with the [`gates::default_pipeline`].
    pub fn new(pool: Pool) -> Self {
        Self {
            pool,
            pipeline: gates::default_pipeline(),
        }
    }

    /// Underlying pool (for advanced callers that need raw access).
    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    /// Plan store accessor.
    pub fn plans(&self) -> PlanStore {
        PlanStore::new(self.pool.clone())
    }

    /// Task store accessor.
    pub fn tasks(&self) -> TaskStore {
        TaskStore::new(self.pool.clone())
    }

    /// Evidence store accessor.
    pub fn evidence(&self) -> EvidenceStore {
        EvidenceStore::new(self.pool.clone())
    }

    /// Audit log accessor.
    pub fn audit(&self) -> AuditLog {
        AuditLog::new(self.pool.clone())
    }

    /// Create a plan and write the audit row.
    pub async fn create_plan(&self, input: NewPlan) -> Result<Plan> {
        let plan = self.plans().create(input).await?;
        self.audit()
            .append(
                EntityKind::Plan,
                &plan.id,
                "plan.created",
                &json!({
                    "plan_id": plan.id,
                    "title": plan.title,
                }),
                None,
            )
            .await?;
        Ok(plan)
    }

    /// Create a task and write the audit row.
    pub async fn create_task(&self, plan_id: &str, input: NewTask) -> Result<Task> {
        // Make sure the plan exists (yields NotFound if not).
        self.plans().get(plan_id).await?;
        let task = self.tasks().create(plan_id, input).await?;
        self.audit()
            .append(
                EntityKind::Task,
                &task.id,
                "task.created",
                &json!({
                    "task_id": task.id,
                    "plan_id": task.plan_id,
                    "wave": task.wave,
                    "sequence": task.sequence,
                    "title": task.title,
                }),
                None,
            )
            .await?;
        Ok(task)
    }

    /// Move a task to a new status, running the gate pipeline first.
    /// On success, writes one audit row.
    pub async fn transition_task(
        &self,
        task_id: &str,
        target: TaskStatus,
        agent_id: Option<&str>,
    ) -> Result<Task> {
        let task = self.tasks().get(task_id).await?;
        let ctx = GateContext {
            pool: self.pool.clone(),
            task: task.clone(),
            target_status: target,
            agent_id: agent_id.map(str::to_string),
        };
        gates::run(&self.pipeline, &ctx).await?;

        self.tasks().set_status(task_id, target, agent_id).await?;
        self.audit()
            .append(
                EntityKind::Task,
                task_id,
                &format!("task.{}", target.as_str()),
                &json!({
                    "task_id": task_id,
                    "from": task.status.as_str(),
                    "to": target.as_str(),
                    "agent_id": agent_id,
                }),
                agent_id,
            )
            .await?;
        self.tasks().get(task_id).await
    }

    /// Attach evidence to a task and write the audit row.
    pub async fn attach_evidence(
        &self,
        task_id: &str,
        kind: &str,
        payload: serde_json::Value,
        exit_code: Option<i64>,
    ) -> Result<crate::model::Evidence> {
        // Confirm task exists.
        self.tasks().get(task_id).await?;
        let evidence = self
            .evidence()
            .attach(task_id, kind, payload, exit_code)
            .await?;
        self.audit()
            .append(
                EntityKind::Evidence,
                &evidence.id,
                "evidence.attached",
                &json!({
                    "evidence_id": evidence.id,
                    "task_id": task_id,
                    "kind": kind,
                    "exit_code": exit_code,
                }),
                None,
            )
            .await?;
        Ok(evidence)
    }
}
