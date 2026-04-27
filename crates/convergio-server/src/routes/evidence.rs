//! `/v1/tasks/:id/evidence` — attach + list.

use crate::app::AppState;
use crate::error::ApiError;
use axum::extract::{Path, State};
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::Value;

/// Mount evidence routes.
pub fn router() -> Router<AppState> {
    Router::new().route("/v1/tasks/:id/evidence", post(attach).get(list))
}

#[derive(Deserialize)]
struct AttachBody {
    kind: String,
    payload: Value,
    #[serde(default)]
    exit_code: Option<i64>,
}

async fn attach(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<AttachBody>,
) -> Result<Json<convergio_durability::Evidence>, ApiError> {
    let evidence = state
        .durability
        .attach_evidence(&id, &body.kind, body.payload, body.exit_code)
        .await?;
    Ok(Json(evidence))
}

async fn list(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<convergio_durability::Evidence>>, ApiError> {
    let evidence = state.durability.evidence().list_by_task(&id).await?;
    Ok(Json(evidence))
}
