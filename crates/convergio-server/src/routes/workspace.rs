//! `/v1/workspace/*` coordination routes.

use crate::app::AppState;
use crate::error::ApiError;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use convergio_durability::{
    NewPatchProposal, NewWorkspaceLease, PatchProposal, WorkspaceConflict, WorkspaceLease,
};

/// Mount workspace coordination routes.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/workspace/leases", get(active_leases).post(claim_lease))
        .route("/v1/workspace/leases/:id/release", post(release_lease))
        .route("/v1/workspace/patches", post(submit_patch))
        .route("/v1/workspace/conflicts", get(open_conflicts))
}

async fn active_leases(
    State(state): State<AppState>,
) -> Result<Json<Vec<WorkspaceLease>>, ApiError> {
    let leases = state.durability.workspace().active_leases().await?;
    Ok(Json(leases))
}

async fn claim_lease(
    State(state): State<AppState>,
    Json(body): Json<NewWorkspaceLease>,
) -> Result<Json<WorkspaceLease>, ApiError> {
    let lease = state.durability.workspace().claim_lease(body).await?;
    Ok(Json(lease))
}

async fn release_lease(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<WorkspaceLease>, ApiError> {
    let lease = state.durability.workspace().release_lease(&id).await?;
    Ok(Json(lease))
}

async fn submit_patch(
    State(state): State<AppState>,
    Json(body): Json<NewPatchProposal>,
) -> Result<Json<PatchProposal>, ApiError> {
    let proposal = state.durability.submit_patch_proposal(body).await?;
    Ok(Json(proposal))
}

async fn open_conflicts(
    State(state): State<AppState>,
) -> Result<Json<Vec<WorkspaceConflict>>, ApiError> {
    let conflicts = state
        .durability
        .workspace()
        .open_workspace_conflicts()
        .await?;
    Ok(Json(conflicts))
}
