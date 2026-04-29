//! `/v1/crdt/*` read-only CRDT diagnostics.

use crate::app::AppState;
use crate::error::ApiError;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use convergio_durability::{CrdtCell, CrdtImportResult, NewCrdtOp};
use serde::Deserialize;

/// Mount CRDT diagnostic routes.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/crdt/conflicts", get(conflicts))
        .route("/v1/crdt/import", axum::routing::post(import))
}

#[derive(Deserialize)]
struct ImportBody {
    ops: Vec<NewCrdtOp>,
    #[serde(default)]
    agent_id: Option<String>,
}

async fn conflicts(State(state): State<AppState>) -> Result<Json<Vec<CrdtCell>>, ApiError> {
    let conflicts = state.durability.crdt().list_conflicts().await?;
    Ok(Json(conflicts))
}

async fn import(
    State(state): State<AppState>,
    Json(body): Json<ImportBody>,
) -> Result<Json<CrdtImportResult>, ApiError> {
    let result = state
        .durability
        .import_crdt_ops(body.ops, body.agent_id.as_deref())
        .await?;
    Ok(Json(result))
}
