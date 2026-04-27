//! `GET /v1/health` — liveness + version probe.

use crate::app::AppState;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};

/// Mount the health route.
pub fn router() -> Router<AppState> {
    Router::new().route("/v1/health", get(health))
}

async fn health() -> Json<Value> {
    Json(json!({
        "ok": true,
        "service": "convergio",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
