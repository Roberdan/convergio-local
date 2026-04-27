//! `POST /v1/plans/:id/validate` — Thor verdict.

use crate::app::AppState;
use crate::error::ApiError;
use axum::extract::{Path, State};
use axum::routing::post;
use axum::{Json, Router};
use convergio_thor::{Thor, Verdict};

/// Mount the validate route.
pub fn router() -> Router<AppState> {
    Router::new().route("/v1/plans/:id/validate", post(validate))
}

async fn validate(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Verdict>, ApiError> {
    let thor = Thor::new((*state.durability).clone());
    let verdict = thor.validate(&id).await?;
    Ok(Json(verdict))
}
