//! Audited workspace coordination facade operations.

use crate::audit::EntityKind;
use crate::store::{NewPatchProposal, PatchProposal};
use crate::{Durability, DurabilityError, Result};
use serde_json::json;
use uuid::Uuid;

impl Durability {
    /// Submit a workspace patch proposal and audit accept/refuse outcomes.
    pub async fn submit_patch_proposal(&self, input: NewPatchProposal) -> Result<PatchProposal> {
        let task_id = input.task_id.clone();
        let agent_id = input.agent_id.clone();
        match self.workspace().submit_patch_proposal(input).await {
            Ok(proposal) => {
                self.audit_patch_proposed(&proposal).await?;
                Ok(proposal)
            }
            Err(err) => {
                if let DurabilityError::WorkspacePatchRefused { kind, reason } = &err {
                    self.audit()
                        .append(
                            EntityKind::Workspace,
                            &Uuid::new_v4().to_string(),
                            "workspace.patch_refused",
                            &json!({
                                "task_id": task_id,
                                "agent_id": agent_id,
                                "kind": kind,
                                "reason": reason,
                            }),
                            Some(&agent_id),
                        )
                        .await?;
                }
                Err(err)
            }
        }
    }

    async fn audit_patch_proposed(&self, proposal: &PatchProposal) -> Result<()> {
        self.audit()
            .append(
                EntityKind::Workspace,
                &proposal.id,
                "workspace.patch_proposed",
                &json!({
                    "proposal_id": proposal.id,
                    "task_id": proposal.task_id,
                    "agent_id": proposal.agent_id,
                    "base_revision": proposal.base_revision,
                    "files": proposal.files.iter().map(|file| &file.path).collect::<Vec<_>>(),
                }),
                Some(&proposal.agent_id),
            )
            .await?;
        Ok(())
    }
}
