//! `Durability` — the public facade tying stores, gates and audit log
//! together so that callers (HTTP layer, CLI) only see one type.

use crate::audit::{append_tx, AuditLog, EntityKind};
use crate::error::{DurabilityError, Result};
use crate::gates::{self, GateContext, Pipeline};
use crate::model::{Evidence, NewPlan, NewTask, Plan, PlanStatus, Task, TaskStatus};
use crate::store::{CrdtStore, EvidenceStore, PlanStore, TaskStore, WorkspaceStore};
use chrono::Utc;
use convergio_db::Pool;
use serde_json::json;
use uuid::Uuid;

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

    /// CRDT actor/op store accessor.
    pub fn crdt(&self) -> CrdtStore {
        CrdtStore::new(self.pool.clone())
    }

    /// Workspace coordination store accessor.
    pub fn workspace(&self) -> WorkspaceStore {
        WorkspaceStore::new(self.pool.clone())
    }

    /// Audit log accessor.
    pub fn audit(&self) -> AuditLog {
        AuditLog::new(self.pool.clone())
    }

    /// Create a plan and write the audit row.
    pub async fn create_plan(&self, input: NewPlan) -> Result<Plan> {
        let now = Utc::now();
        let plan = Plan {
            id: Uuid::new_v4().to_string(),
            title: input.title,
            description: input.description,
            project: input.project,
            status: PlanStatus::Draft,
            created_at: now,
            updated_at: now,
        };

        let mut tx = self.pool.inner().begin().await?;
        sqlx::query(
            "INSERT INTO plans (id, title, description, project, status, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&plan.id)
        .bind(&plan.title)
        .bind(&plan.description)
        .bind(&plan.project)
        .bind(plan.status.as_str())
        .bind(plan.created_at.to_rfc3339())
        .bind(plan.updated_at.to_rfc3339())
        .execute(&mut *tx)
        .await?;
        append_tx(
            &mut tx,
            EntityKind::Plan,
            &plan.id,
            "plan.created",
            &json!({
                "plan_id": plan.id,
                "title": plan.title,
                "project": plan.project,
            }),
            None,
        )
        .await?;
        tx.commit().await?;
        Ok(plan)
    }

    /// Create a task and write the audit row.
    pub async fn create_task(&self, plan_id: &str, input: NewTask) -> Result<Task> {
        // Make sure the plan exists (yields NotFound if not).
        self.plans().get(plan_id).await?;
        let now = Utc::now();
        let task = Task {
            id: Uuid::new_v4().to_string(),
            plan_id: plan_id.to_string(),
            wave: input.wave,
            sequence: input.sequence,
            title: input.title,
            description: input.description,
            status: TaskStatus::Pending,
            agent_id: None,
            evidence_required: input.evidence_required,
            last_heartbeat_at: None,
            created_at: now,
            updated_at: now,
        };

        let mut tx = self.pool.inner().begin().await?;
        sqlx::query(
            "INSERT INTO tasks (id, plan_id, wave, sequence, title, description, status, \
             agent_id, evidence_required, last_heartbeat_at, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&task.id)
        .bind(&task.plan_id)
        .bind(task.wave)
        .bind(task.sequence)
        .bind(&task.title)
        .bind(&task.description)
        .bind(task.status.as_str())
        .bind(&task.agent_id)
        .bind(serde_json::to_string(&task.evidence_required)?)
        .bind(Option::<String>::None)
        .bind(task.created_at.to_rfc3339())
        .bind(task.updated_at.to_rfc3339())
        .execute(&mut *tx)
        .await?;
        append_tx(
            &mut tx,
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
        tx.commit().await?;
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
        if let Err(e) = gates::run(&self.pipeline, &ctx).await {
            if let DurabilityError::GateRefused { gate, reason } = &e {
                self.record_gate_refusal(&task, target, agent_id, gate, reason)
                    .await?;
            }
            return Err(e);
        }

        let mut tx = self.pool.inner().begin().await?;
        sqlx::query("UPDATE tasks SET status = ?, agent_id = ?, updated_at = ? WHERE id = ?")
            .bind(target.as_str())
            .bind(agent_id)
            .bind(Utc::now().to_rfc3339())
            .bind(task_id)
            .execute(&mut *tx)
            .await?;
        append_tx(
            &mut tx,
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
        tx.commit().await?;
        self.tasks().get(task_id).await
    }

    async fn record_gate_refusal(
        &self,
        task: &Task,
        target: TaskStatus,
        agent_id: Option<&str>,
        gate: &str,
        reason: &str,
    ) -> Result<()> {
        self.audit()
            .append(
                EntityKind::Task,
                &task.id,
                "task.refused",
                &json!({
                    "task_id": task.id,
                    "from": task.status.as_str(),
                    "to": target.as_str(),
                    "gate": gate,
                    "reason": reason,
                    "agent_id": agent_id,
                }),
                agent_id,
            )
            .await?;
        Ok(())
    }

    /// Attach evidence to a task and write the audit row.
    pub async fn attach_evidence(
        &self,
        task_id: &str,
        kind: &str,
        payload: serde_json::Value,
        exit_code: Option<i64>,
    ) -> Result<Evidence> {
        // Confirm task exists.
        self.tasks().get(task_id).await?;
        let evidence = Evidence {
            id: Uuid::new_v4().to_string(),
            task_id: task_id.to_string(),
            kind: kind.to_string(),
            payload,
            exit_code,
            created_at: Utc::now(),
        };

        let mut tx = self.pool.inner().begin().await?;
        sqlx::query(
            "INSERT INTO evidence (id, task_id, kind, payload, exit_code, created_at) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&evidence.id)
        .bind(&evidence.task_id)
        .bind(&evidence.kind)
        .bind(serde_json::to_string(&evidence.payload)?)
        .bind(evidence.exit_code)
        .bind(evidence.created_at.to_rfc3339())
        .execute(&mut *tx)
        .await?;
        append_tx(
            &mut tx,
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
        tx.commit().await?;
        Ok(evidence)
    }
}
