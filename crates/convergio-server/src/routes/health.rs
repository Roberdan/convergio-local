//! `GET /v1/health` — liveness + version probe.
//!
//! The response carries the daemon's running version. If the operator
//! exports `CONVERGIO_EXPECTED_VERSION` (the post-merge `cvg update`
//! flow does this when restarting the daemon), the response also
//! includes `expected_version` and a `drift` boolean so the CLI can
//! warn the user when the workspace has moved past the running
//! daemon. F50 — closes the dev-loop case where the daemon keeps
//! running an older binary after a `git pull`.

use crate::app::AppState;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};

/// Mount the health route.
pub fn router() -> Router<AppState> {
    Router::new().route("/v1/health", get(health))
}

async fn health() -> Json<Value> {
    let expected = std::env::var("CONVERGIO_EXPECTED_VERSION")
        .ok()
        .filter(|v| !v.trim().is_empty());
    Json(build_health_body(
        env!("CARGO_PKG_VERSION"),
        expected.as_deref(),
    ))
}

fn build_health_body(running: &str, expected: Option<&str>) -> Value {
    let drift = matches!(expected, Some(v) if v != running);
    json!({
        "ok": true,
        "service": "convergio",
        "version": running,
        "running_version": running,
        "expected_version": expected,
        "drift": drift,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_no_drift_when_expected_is_none() {
        let body = build_health_body("1.2.3", None);
        assert_eq!(body["ok"], true);
        assert_eq!(body["service"], "convergio");
        assert_eq!(body["version"], "1.2.3");
        assert_eq!(body["running_version"], "1.2.3");
        assert_eq!(body["expected_version"], serde_json::Value::Null);
        assert_eq!(body["drift"], false);
    }

    #[test]
    fn health_drift_when_expected_mismatches_running() {
        let body = build_health_body("1.2.3", Some("9.9.9"));
        assert_eq!(body["expected_version"], "9.9.9");
        assert_eq!(body["drift"], true);
    }

    #[test]
    fn health_no_drift_when_expected_matches_running() {
        let body = build_health_body("1.2.3", Some("1.2.3"));
        assert_eq!(body["expected_version"], "1.2.3");
        assert_eq!(body["drift"], false);
    }
}
