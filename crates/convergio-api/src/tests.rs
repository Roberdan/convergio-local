//! Unit tests for the shared action contract.

use super::*;

#[test]
fn action_names_are_stable_snake_case() {
    let names: Vec<&str> = Action::ALL.iter().map(|a| a.as_str()).collect();
    assert_eq!(names[0], "status");
    assert!(names.contains(&"submit_task"));
    assert!(names.contains(&"list_crdt_conflicts"));
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
