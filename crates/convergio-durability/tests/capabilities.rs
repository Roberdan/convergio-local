//! Capability registry tests.

use convergio_db::Pool;
use convergio_durability::{init, Durability, DurabilityError, NewCapability};
use serde_json::json;
use tempfile::TempDir;

async fn fresh() -> (Durability, TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("state.db");
    let pool = Pool::connect(&format!("sqlite://{}", db.display()))
        .await
        .unwrap();
    init(&pool).await.unwrap();
    (Durability::new(pool), dir)
}

fn planner() -> NewCapability {
    NewCapability {
        name: "planner".into(),
        version: "0.1.0".into(),
        status: "disabled".into(),
        source: "local".into(),
        root_path: Some("/tmp/planner".into()),
        manifest: json!({"actions": ["planner.solve"]}),
        checksum: Some("sha256:abc".into()),
        signature: Some("sig".into()),
    }
}

#[tokio::test]
async fn capability_registry_persists_status_and_audit() {
    let (dur, _dir) = fresh().await;
    let cap = dur.register_capability(planner()).await.unwrap();
    assert_eq!(cap.status, "disabled");
    assert_eq!(cap.manifest["actions"][0], "planner.solve");

    let cap = dur
        .set_capability_status("planner", "enabled")
        .await
        .unwrap();
    assert_eq!(cap.status, "enabled");
    let caps = dur.capabilities().list().await.unwrap();
    assert_eq!(caps.len(), 1);
    assert!(dur.audit().verify(None, None).await.unwrap().ok);
}

#[tokio::test]
async fn invalid_capabilities_are_rejected() {
    let (dur, _dir) = fresh().await;
    let mut cap = planner();
    cap.name = "Bad Name".into();
    let err = dur.register_capability(cap).await.unwrap_err();
    assert!(matches!(err, DurabilityError::InvalidCapability { .. }));

    let err = dur
        .set_capability_status("planner", "unknown")
        .await
        .unwrap_err();
    assert!(matches!(err, DurabilityError::InvalidCapability { .. }));
}
