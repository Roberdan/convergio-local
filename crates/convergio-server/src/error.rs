//! HTTP error type — maps domain errors to status codes.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use convergio_bus::BusError;
use convergio_durability::DurabilityError;
use convergio_lifecycle::LifecycleError;
use serde_json::json;

/// API-facing error.
pub enum ApiError {
    /// Layer 1 error.
    Durability(DurabilityError),
    /// Layer 2 error.
    Bus(BusError),
    /// Layer 3 error.
    Lifecycle(LifecycleError),
}

impl From<DurabilityError> for ApiError {
    fn from(e: DurabilityError) -> Self {
        Self::Durability(e)
    }
}

impl From<BusError> for ApiError {
    fn from(e: BusError) -> Self {
        Self::Bus(e)
    }
}

impl From<LifecycleError> for ApiError {
    fn from(e: LifecycleError) -> Self {
        Self::Lifecycle(e)
    }
}

impl From<convergio_planner::PlannerError> for ApiError {
    fn from(e: convergio_planner::PlannerError) -> Self {
        match e {
            convergio_planner::PlannerError::Durability(d) => Self::Durability(d),
            convergio_planner::PlannerError::EmptyMission => {
                Self::Durability(DurabilityError::NotFound {
                    entity: "mission",
                    id: "empty".into(),
                })
            }
        }
    }
}

impl From<convergio_thor::ThorError> for ApiError {
    fn from(e: convergio_thor::ThorError) -> Self {
        match e {
            convergio_thor::ThorError::Durability(d) => Self::Durability(d),
            convergio_thor::ThorError::PlanNotFound(id) => {
                Self::Durability(DurabilityError::NotFound { entity: "plan", id })
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            ApiError::Durability(e) => match e {
                DurabilityError::NotFound { .. } => {
                    (StatusCode::NOT_FOUND, "not_found", e.to_string())
                }
                DurabilityError::GateRefused { gate, reason } => (
                    StatusCode::CONFLICT,
                    "gate_refused",
                    format!("{gate}: {reason}"),
                ),
                DurabilityError::WorkspaceLeaseConflict { .. } => (
                    StatusCode::CONFLICT,
                    "workspace_lease_conflict",
                    e.to_string(),
                ),
                DurabilityError::InvalidWorkspaceLease { .. } => (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "invalid_workspace_lease",
                    e.to_string(),
                ),
                DurabilityError::AuditChainBroken { .. } => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "audit_broken",
                    e.to_string(),
                ),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal", e.to_string()),
            },
            ApiError::Bus(e) => match e {
                BusError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found", e.to_string()),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal", e.to_string()),
            },
            ApiError::Lifecycle(e) => match e {
                LifecycleError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found", e.to_string()),
                LifecycleError::SpawnFailed(_) => (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "spawn_failed",
                    e.to_string(),
                ),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal", e.to_string()),
            },
        };

        let body = json!({
            "error": {
                "code": code,
                "message": message,
            }
        });
        (status, Json(body)).into_response()
    }
}
