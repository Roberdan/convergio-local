//! HTTP error type — maps domain errors to status codes.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use convergio_durability::DurabilityError;
use serde_json::json;

/// API-facing error.
pub struct ApiError(pub DurabilityError);

impl From<DurabilityError> for ApiError {
    fn from(e: DurabilityError) -> Self {
        Self(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self.0 {
            DurabilityError::NotFound { .. } => {
                (StatusCode::NOT_FOUND, "not_found", self.0.to_string())
            }
            DurabilityError::GateRefused { gate, reason } => (
                StatusCode::CONFLICT,
                "gate_refused",
                format!("{gate}: {reason}"),
            ),
            DurabilityError::AuditChainBroken { .. } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "audit_broken",
                self.0.to_string(),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal",
                self.0.to_string(),
            ),
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
