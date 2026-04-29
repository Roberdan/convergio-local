//! `/v1/audit/verify` — recompute the chain.

use crate::app::AppState;
use crate::error::ApiError;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use convergio_durability::audit::{AuditEntry, VerifyReport};
use serde::Deserialize;

/// Mount audit routes.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/audit/verify", get(verify))
        .route("/v1/audit/refusals/latest", get(latest_refusal))
}

#[derive(Deserialize)]
struct VerifyQuery {
    #[serde(default)]
    from: Option<i64>,
    #[serde(default)]
    to: Option<i64>,
}

#[derive(Deserialize)]
struct RefusalQuery {
    #[serde(default)]
    task_id: Option<String>,
}

async fn verify(
    State(state): State<AppState>,
    Query(q): Query<VerifyQuery>,
) -> Result<Json<VerifyReport>, ApiError> {
    let report = state.durability.audit().verify(q.from, q.to).await?;
    Ok(Json(report))
}

async fn latest_refusal(
    State(state): State<AppState>,
    Query(q): Query<RefusalQuery>,
) -> Result<Json<Option<AuditEntry>>, ApiError> {
    let entry = state
        .durability
        .audit()
        .latest_refusal(q.task_id.as_deref())
        .await?;
    Ok(Json(entry))
}
