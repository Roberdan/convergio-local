//! Capability registry tests.

use convergio_db::Pool;
use convergio_durability::{
    capability_signature_payload, init, CapabilitySignatureRequest, Durability, DurabilityError,
    NewCapability, TrustedCapabilityKey,
};
use ed25519_dalek::{Signer, SigningKey};
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

fn signed_capability_request() -> CapabilitySignatureRequest {
    let signing_key = SigningKey::from_bytes(&[7; 32]);
    let mut request = CapabilitySignatureRequest {
        name: "planner".into(),
        version: "0.1.0".into(),
        checksum: format!("sha256:{}", "a".repeat(64)),
        manifest: json!({"actions": ["planner.solve"]}),
        signature: String::new(),
        trusted_keys: vec![TrustedCapabilityKey {
            key_id: "test-root".into(),
            public_key: hex::encode(signing_key.verifying_key().to_bytes()),
        }],
    };
    let payload = capability_signature_payload(&request).unwrap();
    request.signature = hex::encode(signing_key.sign(&payload).to_bytes());
    request
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

#[tokio::test]
async fn capability_signature_verification_accepts_trusted_signature_and_audits() {
    let (dur, _dir) = fresh().await;
    let report = dur
        .verify_capability_signature(signed_capability_request())
        .await
        .unwrap();
    assert_eq!(report.name, "planner");
    assert_eq!(report.key_id, "test-root");
    assert!(dur.audit().verify(None, None).await.unwrap().ok);
}

#[tokio::test]
async fn capability_signature_verification_rejects_bad_signature_and_audits() {
    let (dur, _dir) = fresh().await;
    let mut request = signed_capability_request();
    let mut signature = hex::decode(&request.signature).unwrap();
    signature[0] ^= 0xff;
    request.signature = hex::encode(signature);
    let err = dur.verify_capability_signature(request).await.unwrap_err();
    assert!(matches!(err, DurabilityError::InvalidCapability { .. }));
    assert!(dur.audit().verify(None, None).await.unwrap().ok);
}

#[tokio::test]
async fn capability_signature_verification_rejects_missing_signature() {
    let (dur, _dir) = fresh().await;
    let mut request = signed_capability_request();
    request.signature.clear();
    let err = dur.verify_capability_signature(request).await.unwrap_err();
    assert!(matches!(err, DurabilityError::InvalidCapability { .. }));
}

#[tokio::test]
async fn capability_signature_verification_rejects_untrusted_key() {
    let (dur, _dir) = fresh().await;
    let mut request = signed_capability_request();
    request.trusted_keys.clear();
    let err = dur.verify_capability_signature(request).await.unwrap_err();
    assert!(matches!(err, DurabilityError::InvalidCapability { .. }));
}
