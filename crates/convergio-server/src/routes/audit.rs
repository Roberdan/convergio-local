//! `/v1/audit/verify` — recompute the chain.

use crate::app::AppState;
use crate::error::ApiError;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use convergio_durability::audit::VerifyReport;
use serde::Deserialize;

/// Mount audit routes.
pub fn router() -> Router<AppState> {
    Router::new().route("/v1/audit/verify", get(verify))
}

#[derive(Deserialize)]
struct VerifyQuery {
    #[serde(default)]
    from: Option<i64>,
    #[serde(default)]
    to: Option<i64>,
}

async fn verify(
    State(state): State<AppState>,
    Query(q): Query<VerifyQuery>,
) -> Result<Json<VerifyReport>, ApiError> {
    let report = state.durability.audit().verify(q.from, q.to).await?;
    Ok(Json(report))
}
