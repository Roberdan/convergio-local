//! `POST /v1/solve` — turn a mission into a plan via Layer 4 planner.

use crate::app::AppState;
use crate::error::ApiError;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use convergio_planner::Planner;
use serde::Deserialize;
use serde_json::{json, Value};

/// Mount the solve route.
pub fn router() -> Router<AppState> {
    Router::new().route("/v1/solve", post(solve))
}

#[derive(Deserialize)]
struct SolveBody {
    mission: String,
}

async fn solve(
    State(state): State<AppState>,
    Json(body): Json<SolveBody>,
) -> Result<Json<Value>, ApiError> {
    let planner = Planner::new((*state.durability).clone());
    let plan_id = planner.solve(&body.mission).await?;
    Ok(Json(json!({"plan_id": plan_id})))
}
