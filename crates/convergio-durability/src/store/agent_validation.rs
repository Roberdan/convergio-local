//! Validation helpers for durable agent registry inputs.

use crate::error::{DurabilityError, Result};

pub(super) fn validate_agent_id(id: &str) -> Result<()> {
    if id.trim().is_empty() || id.contains(char::is_whitespace) {
        return Err(DurabilityError::InvalidAgent {
            reason: "agent id must be non-empty and contain no whitespace".into(),
        });
    }
    Ok(())
}

pub(super) fn validate_agent_kind(kind: &str) -> Result<()> {
    if kind.is_empty() {
        return Err(DurabilityError::InvalidAgent {
            reason: "agent kind must be non-empty".into(),
        });
    }
    if kind.len() > 64 {
        return Err(DurabilityError::InvalidAgent {
            reason: format!("agent kind too long ({} > 64 chars)", kind.len()),
        });
    }
    if kind
        .chars()
        .any(|c| !(c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '.' | '_')))
    {
        return Err(DurabilityError::InvalidAgent {
            reason: format!(
                "agent kind '{kind}' has invalid characters \
                 (allowed: lower-case ASCII letters, digits, '-', '.', '_')"
            ),
        });
    }
    Ok(())
}

pub(super) fn validate_status(status: &str) -> Result<()> {
    if !matches!(status, "idle" | "working" | "unhealthy" | "terminated") {
        return Err(DurabilityError::InvalidAgent {
            reason: "unknown agent status".into(),
        });
    }
    Ok(())
}
