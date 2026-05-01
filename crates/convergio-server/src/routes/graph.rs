//! `/v1/graph/*` — Tier-3 code-graph endpoints (ADR-0014).
//!
//! v0 surfaced `build` + `stats`. PR 14.2 adds `for-task` (the
//! context-pack query) and `refresh` (lefthook nudge). `cluster` +
//! `drift` land in PR 14.3.

use crate::app::AppState;
use crate::error::ApiError;
use axum::extract::{Path as AxumPath, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use convergio_graph::{
    BuildReport, ContextPack, DriftReport, DEFAULT_NODE_LIMIT, DEFAULT_TOKEN_BUDGET,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Mount the graph routes.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/graph/build", post(build))
        .route("/v1/graph/stats", get(stats))
        .route("/v1/graph/refresh", post(refresh))
        .route("/v1/graph/for-task/:id", get(for_task))
        .route("/v1/graph/drift", get(drift))
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

#[derive(Debug, Deserialize, Default)]
struct ForTaskQuery {
    /// Override the default node-count cap.
    #[serde(default)]
    node_limit: Option<usize>,
    /// Override the default token budget.
    #[serde(default)]
    token_budget: Option<u64>,
}

/// `GET /v1/graph/for-task/:id` — context-pack for the named task.
async fn for_task(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<String>,
    Query(q): Query<ForTaskQuery>,
) -> Result<Json<ContextPack>, ApiError> {
    let task = state.durability.tasks().get(&id).await?;
    let text = format!(
        "{}\n{}",
        task.title,
        task.description.as_deref().unwrap_or("")
    );
    let pack = convergio_graph::for_task_text(
        &state.graph,
        &task.id,
        &text,
        q.node_limit.unwrap_or(DEFAULT_NODE_LIMIT),
        q.token_budget.unwrap_or(DEFAULT_TOKEN_BUDGET),
    )
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(pack))
}

/// `POST /v1/graph/refresh` — lefthook nudge after a commit.
/// Re-runs an incremental build against the daemon's cwd. Returns
/// the build report so a caller can verify what changed. Idempotent.
async fn refresh(State(state): State<AppState>) -> Result<Json<BuildReport>, ApiError> {
    let manifest = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let report = convergio_graph::build(&manifest, &state.graph, false)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(report))
}

#[derive(Debug, Deserialize, Default)]
struct DriftQuery {
    /// Repo root for the git diff (defaults to daemon cwd).
    #[serde(default)]
    repo_root: Option<String>,
    /// Git ref to compare against (default `origin/main`).
    #[serde(default)]
    since: Option<String>,
    /// Optional ADR id to scope the declared set to a single ADR.
    #[serde(default)]
    adr: Option<String>,
}

/// `GET /v1/graph/drift` — ADR claims vs git diff (advisory).
async fn drift(
    State(state): State<AppState>,
    Query(q): Query<DriftQuery>,
) -> Result<Json<DriftReport>, ApiError> {
    let root = q
        .repo_root
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let since = q.since.as_deref().unwrap_or("origin/main");
    let report = convergio_graph::drift_since(&state.graph, &root, since, q.adr.as_deref())
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(report))
}
