//! `POST /v1/plans/:id/validate[?wave=N]` — Thor verdict.
//!
//! Optional `wave` query parameter (T3.06) restricts validation to a
//! single wave of the plan. Tasks in other waves are ignored — they
//! neither block the verdict nor get promoted. Without the
//! parameter, validation is plan-strict (default behaviour).

use crate::app::AppState;
use crate::error::ApiError;
use axum::extract::{Path, Query, State};
use axum::routing::post;
use axum::{Json, Router};
use convergio_thor::{Thor, Verdict};
use serde::Deserialize;

/// Mount the validate route.
pub fn router() -> Router<AppState> {
    Router::new().route("/v1/plans/:id/validate", post(validate))
}

/// Query parameters accepted by `validate`.
#[derive(Debug, Default, Deserialize)]
struct ValidateQuery {
    /// Restrict validation to this wave only (T3.06).
    wave: Option<i64>,
}

async fn validate(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<ValidateQuery>,
) -> Result<Json<Verdict>, ApiError> {
    let thor = Thor::new((*state.durability).clone());
    let verdict = thor.validate_wave(&id, q.wave).await?;
    Ok(Json(verdict))
}
