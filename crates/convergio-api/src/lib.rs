//! Shared agent-facing action contract for Convergio integrations.
//!
//! The HTTP daemon remains the source of truth. This crate only defines
//! the compact, versioned request/response shapes used by adapters such
//! as the MCP bridge.

#![forbid(unsafe_code)]

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Current major schema version for agent actions.
pub const SCHEMA_VERSION: &str = "1";

/// Stable MCP help tool name.
pub const HELP_TOOL: &str = "convergio.help";

/// Stable MCP action tool name.
pub const ACT_TOOL: &str = "convergio.act";

/// Action capabilities exposed to agents.
pub const CAPABILITIES: &[&str] = &[
    "status",
    "plans",
    "tasks",
    "evidence",
    "audit",
    "validation",
];

/// Closed set of task-oriented actions accepted by `convergio.act`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// Diagnose daemon and integration readiness.
    Status,
    /// Create a plan.
    CreatePlan,
    /// Create a task under a plan.
    CreateTask,
    /// List tasks for a plan.
    ListTasks,
    /// Find the next task an agent should work on.
    NextTask,
    /// Claim a task as in progress.
    ClaimTask,
    /// Touch a task heartbeat.
    Heartbeat,
    /// Attach evidence to a task.
    AddEvidence,
    /// Submit a task and run gates.
    SubmitTask,
    /// Mark a submitted task done.
    CompleteTask,
    /// Validate a plan.
    ValidatePlan,
    /// Verify the audit hash chain.
    AuditVerify,
    /// Explain the most recent gate refusal for a task.
    ExplainLastRefusal,
    /// Return the canonical prompt addendum for agents.
    AgentPrompt,
}

impl Action {
    /// Every supported action in stable display order.
    pub const ALL: &'static [Self] = &[
        Self::Status,
        Self::CreatePlan,
        Self::CreateTask,
        Self::ListTasks,
        Self::NextTask,
        Self::ClaimTask,
        Self::Heartbeat,
        Self::AddEvidence,
        Self::SubmitTask,
        Self::CompleteTask,
        Self::ValidatePlan,
        Self::AuditVerify,
        Self::ExplainLastRefusal,
        Self::AgentPrompt,
    ];

    /// Stable snake_case action name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::CreatePlan => "create_plan",
            Self::CreateTask => "create_task",
            Self::ListTasks => "list_tasks",
            Self::NextTask => "next_task",
            Self::ClaimTask => "claim_task",
            Self::Heartbeat => "heartbeat",
            Self::AddEvidence => "add_evidence",
            Self::SubmitTask => "submit_task",
            Self::CompleteTask => "complete_task",
            Self::ValidatePlan => "validate_plan",
            Self::AuditVerify => "audit_verify",
            Self::ExplainLastRefusal => "explain_last_refusal",
            Self::AgentPrompt => "agent_prompt",
        }
    }
}

/// Read-only help topics served by `convergio.help`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HelpTopic {
    /// Minimal session bootstrap.
    Quickstart,
    /// Action catalog.
    Actions,
    /// Details for one action.
    Action,
    /// Evidence payload conventions.
    EvidenceSchema,
    /// How agents should handle gate refusals.
    GateRefusal,
    /// Local setup instructions.
    Setup,
    /// Canonical agent prompt addendum.
    Prompt,
}

/// Help output verbosity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HelpVerbosity {
    /// Compact default for agent context.
    Short,
    /// Include machine-readable schemas or examples.
    Schema,
    /// Full explanatory content for humans/debugging.
    Full,
}

/// Request accepted by `convergio.help`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HelpRequest {
    /// Requested help topic.
    #[serde(default = "default_help_topic")]
    pub topic: HelpTopic,
    /// Optional action name when `topic` is `Action`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<Action>,
    /// Requested verbosity.
    #[serde(default = "default_help_verbosity")]
    pub verbosity: HelpVerbosity,
}

/// Request accepted by `convergio.act`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ActRequest {
    /// Schema major version the agent used when building this request.
    pub schema_version: String,
    /// Constrained action name.
    pub action: Action,
    /// Action-specific input object.
    #[serde(default)]
    pub params: Value,
}

/// Stable response envelope returned to agents.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentResponse {
    /// Whether the action succeeded.
    pub ok: bool,
    /// Stable machine-readable response code.
    pub code: AgentCode,
    /// Short stable English message.
    pub message: String,
    /// Optional structured payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    /// Optional next action hint for agents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<NextHint>,
}

/// Stable response codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AgentCode {
    /// Action completed.
    Ok,
    /// Daemon is unreachable.
    DaemonUnavailable,
    /// The action schema version is incompatible.
    SchemaVersionMismatch,
    /// Request shape is invalid.
    InvalidRequest,
    /// A Convergio gate refused the transition.
    GateRefused,
    /// Requested resource was not found.
    NotFound,
    /// Any other daemon or bridge failure.
    Error,
}

/// Stable next-step hints for agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NextHint {
    /// Call `convergio.help` again before retrying.
    RefreshHelp,
    /// Start the local daemon.
    StartDaemon,
    /// Fix the issue, attach new evidence, then retry submit.
    FixAddEvidenceRetrySubmit,
    /// Continue with task completion.
    CompleteTask,
    /// Verify audit state.
    VerifyAudit,
}

/// Compact catalog returned by help implementations.
#[derive(Debug, Clone, Serialize)]
pub struct ActionCatalog {
    /// Current schema version.
    pub schema_version: &'static str,
    /// Stable MCP tool names.
    pub tools: ToolNames,
    /// Available capabilities.
    pub capabilities: &'static [&'static str],
    /// Supported actions.
    pub actions: Vec<&'static str>,
}

/// Stable MCP tool names.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolNames {
    /// Read-only help tool.
    pub help: &'static str,
    /// Action dispatcher tool.
    pub act: &'static str,
}

impl ActionCatalog {
    /// Build the default action catalog.
    pub fn current() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            tools: ToolNames {
                help: HELP_TOOL,
                act: ACT_TOOL,
            },
            capabilities: CAPABILITIES,
            actions: Action::ALL.iter().map(|a| a.as_str()).collect(),
        }
    }
}

fn default_help_topic() -> HelpTopic {
    HelpTopic::Quickstart
}

fn default_help_verbosity() -> HelpVerbosity {
    HelpVerbosity::Short
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_names_are_stable_snake_case() {
        let names: Vec<&str> = Action::ALL.iter().map(|a| a.as_str()).collect();
        assert_eq!(names[0], "status");
        assert!(names.contains(&"submit_task"));
        assert!(names.contains(&"explain_last_refusal"));
    }

    #[test]
    fn action_deserializes_from_snake_case() {
        let action: Action = serde_json::from_str("\"add_evidence\"").unwrap();
        assert_eq!(action, Action::AddEvidence);
    }

    #[test]
    fn act_request_has_version_action_and_params() {
        let req: ActRequest = serde_json::from_value(serde_json::json!({
            "schema_version": "1",
            "action": "status",
            "params": {}
        }))
        .unwrap();
        assert_eq!(req.schema_version, SCHEMA_VERSION);
        assert_eq!(req.action, Action::Status);
    }

    #[test]
    fn catalog_exposes_exact_two_tools() {
        let catalog = ActionCatalog::current();
        assert_eq!(catalog.tools.help, HELP_TOOL);
        assert_eq!(catalog.tools.act, ACT_TOOL);
        assert_eq!(catalog.actions.len(), Action::ALL.len());
    }
}
