//! Unit tests for the shared action contract.

use super::*;

#[test]
fn action_names_are_stable_snake_case() {
    let names: Vec<&str> = Action::ALL.iter().map(|a| a.as_str()).collect();
    assert_eq!(names[0], "status");
    assert!(names.contains(&"submit_task"));
    assert!(names.contains(&"get_task_context"));
    assert!(names.contains(&"publish_message"));
    assert!(names.contains(&"poll_messages"));
    assert!(names.contains(&"ack_message"));
    assert!(names.contains(&"import_crdt_ops"));
    assert!(names.contains(&"list_crdt_conflicts"));
    assert!(names.contains(&"register_agent"));
    assert!(names.contains(&"heartbeat_agent"));
    assert!(names.contains(&"retire_agent"));
    assert!(names.contains(&"spawn_runner"));
    assert!(names.contains(&"planner.solve"));
    assert!(names.contains(&"list_capabilities"));
    assert!(names.contains(&"claim_workspace_lease"));
    assert!(names.contains(&"list_workspace_leases"));
    assert!(names.contains(&"submit_patch_proposal"));
    assert!(names.contains(&"enqueue_patch_proposal"));
    assert!(names.contains(&"process_merge_queue"));
    assert!(names.contains(&"list_merge_queue"));
    assert!(names.contains(&"explain_last_refusal"));
}

#[test]
fn action_deserializes_from_snake_case() {
    let action: Action = serde_json::from_str("\"add_evidence\"").unwrap();
    assert_eq!(action, Action::AddEvidence);
    let action: Action = serde_json::from_str("\"planner.solve\"").unwrap();
    assert_eq!(action, Action::PlannerSolve);
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
