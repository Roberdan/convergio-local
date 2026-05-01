//! `/v1/graph/*` — Tier-3 code-graph endpoints (ADR-0014).
//!
//! v0 surfaces `build` and `stats`. `for-task`, `cluster`, `drift`
//! land in subsequent PRs (14.2, 14.3).

use crate::app::AppState;
use crate::error::ApiError;
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use convergio_graph::BuildReport;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Mount the graph routes.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/graph/build", post(build))
        .route("/v1/graph/stats", get(stats))
}

#[derive(Debug, Deserialize, Default)]
struct BuildRequest {
    /// Workspace manifest dir. Defaults to the daemon's cwd.
    #[serde(default)]
    manifest_dir: Option<String>,
    /// Force re-parse even if mtime is unchanged.
    #[serde(default)]
    force: bool,
}

#[derive(Debug, Serialize)]
struct StatsResponse {
    nodes: usize,
    edges: usize,
}

async fn build(
    State(state): State<AppState>,
    Json(req): Json<BuildRequest>,
) -> Result<Json<BuildReport>, ApiError> {
    let manifest = req
        .manifest_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let report = convergio_graph::build(&manifest, &state.graph, req.force)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(report))
}

async fn stats(State(state): State<AppState>) -> Result<Json<StatsResponse>, ApiError> {
    let nodes = state
        .graph
        .count_nodes()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let edges = state
        .graph
        .count_edges()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(StatsResponse { nodes, edges }))
}
