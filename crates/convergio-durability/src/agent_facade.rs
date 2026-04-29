//! Audited agent registry facade operations.

use crate::audit::EntityKind;
use crate::store::{AgentHeartbeat, AgentRecord, AgentStore, NewAgent};
use crate::{Durability, Result};
use serde_json::json;

impl Durability {
    /// Agent registry store accessor.
    pub fn agents(&self) -> AgentStore {
        AgentStore::new(self.pool().clone())
    }

    /// Register or refresh an agent identity and write an audit row.
    pub async fn register_agent(&self, input: NewAgent) -> Result<AgentRecord> {
        let agent = self.agents().register(input).await?;
        self.audit()
            .append(
                EntityKind::Agent,
                &agent.id,
                "agent.registered",
                &json!({
                    "agent_id": agent.id,
                    "kind": agent.kind,
                    "capabilities": agent.capabilities,
                }),
                Some(&agent.id),
            )
            .await?;
        Ok(agent)
    }

    /// Record an agent heartbeat and write an audit row.
    pub async fn heartbeat_agent(
        &self,
        agent_id: &str,
        input: AgentHeartbeat,
    ) -> Result<AgentRecord> {
        let agent = self.agents().heartbeat(agent_id, input).await?;
        self.audit()
            .append(
                EntityKind::Agent,
                &agent.id,
                "agent.heartbeat",
                &json!({
                    "agent_id": agent.id,
                    "status": agent.status,
                    "current_task_id": agent.current_task_id,
                }),
                Some(&agent.id),
            )
            .await?;
        Ok(agent)
    }

    /// Retire an agent identity and write an audit row.
    pub async fn retire_agent(&self, agent_id: &str) -> Result<AgentRecord> {
        let agent = self.agents().retire(agent_id).await?;
        self.audit()
            .append(
                EntityKind::Agent,
                &agent.id,
                "agent.retired",
                &json!({"agent_id": agent.id}),
                Some(&agent.id),
            )
            .await?;
        Ok(agent)
    }
}
