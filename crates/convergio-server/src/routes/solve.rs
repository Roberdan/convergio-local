//! `POST /v1/solve` — turn a mission into a plan via Layer 4 planner.

use crate::app::AppState;
use crate::error::ApiError;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use convergio_durability::DurabilityError;
use convergio_planner::Planner;
use serde::Deserialize;
use serde_json::{json, Value};

/// Mount the solve route.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/solve", post(solve))
        .route("/v1/capabilities/planner/solve", post(planner_solve))
}

#[derive(Deserialize)]
struct SolveBody {
    mission: String,
}

async fn solve(
    State(state): State<AppState>,
    Json(body): Json<SolveBody>,
) -> Result<Json<Value>, ApiError> {
    run_planner(&state, body.mission).await
}

async fn planner_solve(
    State(state): State<AppState>,
    Json(body): Json<SolveBody>,
) -> Result<Json<Value>, ApiError> {
    let cap = state.durability.capabilities().get("planner").await?;
    if !matches!(cap.status.as_str(), "installed" | "enabled") {
        return Err(DurabilityError::InvalidCapability {
            reason: "planner capability is not installed or enabled".into(),
        }
        .into());
    }
    run_planner(&state, body.mission).await
}

async fn run_planner(state: &AppState, mission: String) -> Result<Json<Value>, ApiError> {
    let planner = Planner::new((*state.durability).clone());
    let plan_id = planner.solve(&mission).await?;
    Ok(Json(json!({"plan_id": plan_id})))
}
