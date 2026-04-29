//! `/v1/plans/...` — create, list, get.

use crate::app::AppState;
use crate::error::ApiError;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use convergio_durability::{NewPlan, Plan};
use serde::Deserialize;

/// Mount `/v1/plans` routes.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/plans", post(create).get(list))
        .route("/v1/plans/:id", get(by_id))
}

#[derive(Deserialize)]
struct ListQuery {
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_limit() -> i64 {
    50
}

async fn create(
    State(state): State<AppState>,
    Json(body): Json<NewPlan>,
) -> Result<Json<Plan>, ApiError> {
    let plan = state.durability.create_plan(body).await?;
    Ok(Json(plan))
}

async fn list(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<Plan>>, ApiError> {
    let plans = state.durability.plans().list(q.limit).await?;
    Ok(Json(plans))
}

async fn by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Plan>, ApiError> {
    let plan = state.durability.plans().get(&id).await?;
    Ok(Json(plan))
}
