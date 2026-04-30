//! `/v1/capabilities/*` local capability registry routes.

use crate::app::AppState;
use crate::capability_install::{install_file, InstallFileRequest};
use crate::error::ApiError;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use convergio_durability::{
    Capability, CapabilitySignatureRequest, CapabilitySignatureVerification,
};

/// Mount capability registry routes.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/capabilities", get(list))
        .route("/v1/capabilities/install-file", post(install))
        .route("/v1/capabilities/verify-signature", post(verify_signature))
        .route("/v1/capabilities/:name", get(get_one))
}

async fn list(State(state): State<AppState>) -> Result<Json<Vec<Capability>>, ApiError> {
    Ok(Json(state.durability.capabilities().list().await?))
}

async fn get_one(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Capability>, ApiError> {
    Ok(Json(state.durability.capabilities().get(&name).await?))
}

async fn verify_signature(
    State(state): State<AppState>,
    Json(body): Json<CapabilitySignatureRequest>,
) -> Result<Json<CapabilitySignatureVerification>, ApiError> {
    Ok(Json(
        state.durability.verify_capability_signature(body).await?,
    ))
}

async fn install(
    State(state): State<AppState>,
    Json(body): Json<InstallFileRequest>,
) -> Result<Json<Capability>, ApiError> {
    Ok(Json(install_file(&state.durability, body).await?))
}
