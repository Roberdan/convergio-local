//! Audited capability registry facade operations.

use crate::audit::EntityKind;
use crate::store::{Capability, CapabilityStore, NewCapability};
use crate::{Durability, Result};
use serde_json::json;

impl Durability {
    /// Capability registry store accessor.
    pub fn capabilities(&self) -> CapabilityStore {
        CapabilityStore::new(self.pool().clone())
    }

    /// Register or refresh a capability row and write an audit event.
    pub async fn register_capability(&self, input: NewCapability) -> Result<Capability> {
        let cap = self.capabilities().register(input).await?;
        self.audit()
            .append(
                EntityKind::Capability,
                &cap.name,
                "capability.registered",
                &json!({
                    "name": cap.name,
                    "version": cap.version,
                    "status": cap.status,
                    "source": cap.source,
                }),
                None,
            )
            .await?;
        Ok(cap)
    }

    /// Update capability status and write an audit event.
    pub async fn set_capability_status(&self, name: &str, status: &str) -> Result<Capability> {
        let cap = self.capabilities().set_status(name, status).await?;
        self.audit()
            .append(
                EntityKind::Capability,
                &cap.name,
                "capability.status_changed",
                &json!({
                    "name": cap.name,
                    "status": cap.status,
                }),
                None,
            )
            .await?;
        Ok(cap)
    }
}
