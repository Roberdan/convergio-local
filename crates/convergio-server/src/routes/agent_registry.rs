//! `/v1/agent-registry/*` durable agent identity routes.

use crate::app::AppState;
use crate::error::ApiError;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use convergio_durability::{AgentHeartbeat, AgentRecord, NewAgent};

/// Mount agent registry routes.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/agent-registry/agents", get(list).post(register))
        .route("/v1/agent-registry/agents/:id", get(get_one))
        .route("/v1/agent-registry/agents/:id/heartbeat", post(heartbeat))
        .route("/v1/agent-registry/agents/:id/retire", post(retire))
}

async fn register(
    State(state): State<AppState>,
    Json(body): Json<NewAgent>,
) -> Result<Json<AgentRecord>, ApiError> {
    Ok(Json(state.durability.register_agent(body).await?))
}

async fn list(State(state): State<AppState>) -> Result<Json<Vec<AgentRecord>>, ApiError> {
    Ok(Json(state.durability.agents().list().await?))
}

async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AgentRecord>, ApiError> {
    Ok(Json(state.durability.agents().get(&id).await?))
}

async fn heartbeat(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<AgentHeartbeat>,
) -> Result<Json<AgentRecord>, ApiError> {
    Ok(Json(state.durability.heartbeat_agent(&id, body).await?))
}

async fn retire(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AgentRecord>, ApiError> {
    Ok(Json(state.durability.retire_agent(&id).await?))
}
