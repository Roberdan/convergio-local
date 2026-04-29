//! Compact agent-facing help for the MCP bridge.

use convergio_api::{Action, ActionCatalog, HelpRequest, HelpTopic, SCHEMA_VERSION};
use serde_json::{json, Value};

pub(crate) fn response(request: &HelpRequest) -> Value {
    match request.topic {
        HelpTopic::Quickstart => json!({
            "schema_version": SCHEMA_VERSION,
            "tools": ActionCatalog::current().tools,
            "protocol": [
                "call convergio.help once per session",
                "use convergio.act with schema_version and action",
                "never claim done unless submit_task or complete_task succeeds",
                "on gate_refused, fix issue, add evidence, retry"
            ],
        }),
        HelpTopic::Actions => json!(ActionCatalog::current()),
        HelpTopic::Action => action_help(request.action),
        HelpTopic::EvidenceSchema => json!({
            "evidence_required": "each task lists required evidence kinds",
            "payload": "JSON object; include concise command/output facts, not huge logs",
            "exit_code": "0 for successful command evidence; omit when not applicable",
        }),
        HelpTopic::GateRefusal => json!({
            "flow": [
                "read code/message/data from gate_refused response",
                "fix the root cause",
                "attach new evidence",
                "retry submit_task",
            ],
            "next": "fix_add_evidence_retry_submit",
        }),
        HelpTopic::Setup => json!({
            "install": "scripts/install-local.sh",
            "setup": "cvg setup",
            "start": "convergio start",
            "doctor": "cvg doctor --json",
        }),
        HelpTopic::Prompt => agent_prompt(),
    }
}

pub(crate) fn agent_prompt() -> Value {
    json!({
        "prompt": "Use Convergio as the local source of truth. Call convergio.help once. Use convergio.act for task lifecycle and evidence. If a gate refuses work, fix the reason, attach new evidence, and retry. Do not tell the user work is done until Convergio accepts submit_task or complete_task."
    })
}

fn action_help(action: Option<Action>) -> Value {
    let Some(action) = action else {
        return json!({
            "error": "missing action",
            "example": {"topic": "action", "action": "submit_task"}
        });
    };

    match action {
        Action::Status => json!({"params": {}}),
        Action::CreatePlan => json!({
            "params": {
                "title": "string",
                "description": "string?",
                "project": "string?"
            }
        }),
        Action::CreateTask => json!({
            "params": {
                "plan_id": "uuid",
                "title": "string",
                "description": "string?",
                "wave": "integer?",
                "sequence": "integer?",
                "evidence_required": ["code", "test", "doc"]
            }
        }),
        Action::ListTasks | Action::NextTask => json!({"params": {"plan_id": "uuid"}}),
        Action::ClaimTask | Action::SubmitTask | Action::CompleteTask => json!({
            "params": {"task_id": "uuid", "agent_id": "string?"}
        }),
        Action::Heartbeat => json!({"params": {"task_id": "uuid"}}),
        Action::AddEvidence => json!({
            "params": {
                "task_id": "uuid",
                "kind": "code|test|doc|...",
                "payload": "object",
                "exit_code": "integer?"
            }
        }),
        Action::ValidatePlan => json!({"params": {"plan_id": "uuid"}}),
        Action::AuditVerify => json!({"params": {"from": "integer?", "to": "integer?"}}),
        Action::ImportCrdtOps => json!({
            "params": {
                "agent_id": "string?",
                "ops": [{
                    "actor_id": "string",
                    "counter": "integer",
                    "entity_type": "task",
                    "entity_id": "string",
                    "field_name": "string",
                    "crdt_type": "lww_register|mv_register|or_set",
                    "op_kind": "set|add|remove",
                    "value": "json",
                    "hlc": "string"
                }]
            }
        }),
        Action::ListCrdtConflicts => json!({"params": {}}),
        Action::ExplainLastRefusal => json!({"params": {"task_id": "uuid?"}}),
        Action::AgentPrompt => json!({"params": {}}),
    }
}
