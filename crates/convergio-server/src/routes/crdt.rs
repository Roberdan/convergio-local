//! `/v1/crdt/*` read-only CRDT diagnostics.

use crate::app::AppState;
use crate::error::ApiError;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use convergio_durability::CrdtCell;

/// Mount CRDT diagnostic routes.
pub fn router() -> Router<AppState> {
    Router::new().route("/v1/crdt/conflicts", get(conflicts))
}

async fn conflicts(State(state): State<AppState>) -> Result<Json<Vec<CrdtCell>>, ApiError> {
    let conflicts = state.durability.crdt().list_conflicts().await?;
    Ok(Json(conflicts))
}
