//! `/v1/agents/...` — Layer 3 process supervision.

use crate::app::AppState;
use crate::error::ApiError;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use convergio_lifecycle::{AgentProcess, SpawnSpec};

/// Mount Layer 3 routes.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/agents/spawn", post(spawn))
        .route("/v1/agents/:id", get(get_one))
        .route("/v1/agents/:id/heartbeat", post(heartbeat))
}

async fn spawn(
    State(state): State<AppState>,
    Json(spec): Json<SpawnSpec>,
) -> Result<Json<AgentProcess>, ApiError> {
    let proc = state.supervisor.spawn(spec).await?;
    Ok(Json(proc))
}

async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AgentProcess>, ApiError> {
    Ok(Json(state.supervisor.get(&id).await?))
}

async fn heartbeat(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state.supervisor.heartbeat(&id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}
