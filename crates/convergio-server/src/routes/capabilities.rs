//! `/v1/capabilities/*` local capability registry routes.

use crate::app::AppState;
use crate::error::ApiError;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use convergio_durability::Capability;

/// Mount capability registry routes.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/capabilities", get(list))
        .route("/v1/capabilities/:name", get(get_one))
}

async fn list(State(state): State<AppState>) -> Result<Json<Vec<Capability>>, ApiError> {
    Ok(Json(state.durability.capabilities().list().await?))
}

async fn get_one(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Capability>, ApiError> {
    Ok(Json(state.durability.capabilities().get(&name).await?))
}
