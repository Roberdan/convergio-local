//! Audited capability registry facade operations.

use crate::audit::EntityKind;
use crate::capability_signature::{
    verify_capability_signature, CapabilitySignatureRequest, CapabilitySignatureVerification,
};
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

    /// Remove a disabled capability registry row and write an audit event.
    pub async fn remove_capability(&self, name: &str) -> Result<Capability> {
        let cap = self.capabilities().remove(name).await?;
        self.audit()
            .append(
                EntityKind::Capability,
                &cap.name,
                "capability.removed",
                &json!({
                    "name": cap.name,
                    "version": cap.version,
                    "status": cap.status,
                }),
                None,
            )
            .await?;
        Ok(cap)
    }

    /// Verify a capability package signature and write an audit event.
    pub async fn verify_capability_signature(
        &self,
        input: CapabilitySignatureRequest,
    ) -> Result<CapabilitySignatureVerification> {
        match verify_capability_signature(&input) {
            Ok(report) => {
                self.audit_signature_result(
                    &input.name,
                    "capability.signature_verified",
                    &json!({
                        "name": report.name,
                        "version": report.version,
                        "checksum": report.checksum,
                        "key_id": report.key_id,
                        "manifest_sha256": report.manifest_sha256,
                        "payload_sha256": report.payload_sha256,
                    }),
                )
                .await?;
                Ok(report)
            }
            Err(err) => {
                let reason = err.to_string();
                self.audit_signature_result(
                    capability_entity_id(&input.name),
                    "capability.signature_refused",
                    &json!({
                        "name": input.name,
                        "version": input.version,
                        "checksum": input.checksum,
                        "reason": reason,
                    }),
                )
                .await?;
                Err(err)
            }
        }
    }

    async fn audit_signature_result(
        &self,
        entity_id: &str,
        transition: &'static str,
        payload: &serde_json::Value,
    ) -> Result<()> {
        self.audit()
            .append(EntityKind::Capability, entity_id, transition, payload, None)
            .await?;
        Ok(())
    }
}

fn capability_entity_id(name: &str) -> &str {
    if name.is_empty() {
        "unknown"
    } else {
        name
    }
}
