//! `RunnerKind` and parsing.
//!
//! Wire format is `<vendor>:<model>` — e.g. `claude:sonnet`,
//! `claude:opus`, `copilot:gpt-5.2`, `copilot:claude-opus`. The
//! string round-trips through `Display` + `FromStr` so it is safe
//! to store in the `agents.kind` column without a lookup table.

use crate::error::{Result, RunnerError};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Vendor family that owns the CLI binary on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Family {
    /// Anthropic Claude Code CLI (`claude -p ...`).
    Claude,
    /// GitHub Copilot CLI (`copilot -p ...` aka `gh copilot`).
    Copilot,
}

impl Family {
    /// Binary name expected on `PATH`.
    pub fn cli(self) -> &'static str {
        match self {
            Family::Claude => "claude",
            Family::Copilot => "copilot",
        }
    }

    /// String tag used in the `<vendor>:<model>` wire format.
    pub fn tag(self) -> &'static str {
        match self {
            Family::Claude => "claude",
            Family::Copilot => "copilot",
        }
    }
}

/// Concrete runner = vendor + model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerKind {
    /// Vendor CLI family.
    pub family: Family,
    /// Model passed via `--model`. Vendor-specific naming
    /// (`sonnet` / `opus` for Claude, `gpt-5.2` / `claude-opus` for
    /// Copilot). Unknown strings are forwarded as-is so new models
    /// surface without a Convergio release.
    pub model: String,
}

impl RunnerKind {
    /// Build an explicit kind without going through the wire string.
    pub fn new(family: Family, model: impl Into<String>) -> Self {
        Self {
            family,
            model: model.into(),
        }
    }

    /// Default Claude (sonnet) — the cheap, fast option.
    pub fn claude_sonnet() -> Self {
        Self::new(Family::Claude, "sonnet")
    }

    /// Claude Opus — for tasks the planner flags as harder.
    pub fn claude_opus() -> Self {
        Self::new(Family::Claude, "opus")
    }

    /// Copilot with GitHub's default GPT model.
    pub fn copilot_gpt() -> Self {
        Self::new(Family::Copilot, "gpt-5.2")
    }
}

impl fmt::Display for RunnerKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.family.tag(), self.model)
    }
}

impl FromStr for RunnerKind {
    type Err = RunnerError;
    fn from_str(s: &str) -> Result<Self> {
        let (vendor, model) = s
            .split_once(':')
            .ok_or_else(|| RunnerError::InvalidKind(s.to_string()))?;
        let family = match vendor {
            "claude" => Family::Claude,
            "copilot" => Family::Copilot,
            _ => return Err(RunnerError::InvalidKind(s.to_string())),
        };
        if model.is_empty() {
            return Err(RunnerError::InvalidKind(s.to_string()));
        }
        Ok(Self::new(family, model))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_claude_sonnet() {
        let k: RunnerKind = "claude:sonnet".parse().unwrap();
        assert_eq!(k.family, Family::Claude);
        assert_eq!(k.model, "sonnet");
        assert_eq!(k.to_string(), "claude:sonnet");
    }

    #[test]
    fn round_trip_copilot_gpt() {
        let k: RunnerKind = "copilot:gpt-5.2".parse().unwrap();
        assert_eq!(k.family, Family::Copilot);
        assert_eq!(k.model, "gpt-5.2");
    }

    #[test]
    fn unknown_vendor_is_rejected() {
        let err = "openai:gpt-4".parse::<RunnerKind>().unwrap_err();
        assert!(matches!(err, RunnerError::InvalidKind(_)));
    }

    #[test]
    fn empty_model_is_rejected() {
        let err = "claude:".parse::<RunnerKind>().unwrap_err();
        assert!(matches!(err, RunnerError::InvalidKind(_)));
    }

    #[test]
    fn missing_separator_is_rejected() {
        let err = "claude".parse::<RunnerKind>().unwrap_err();
        assert!(matches!(err, RunnerError::InvalidKind(_)));
    }

    #[test]
    fn family_cli_is_stable() {
        assert_eq!(Family::Claude.cli(), "claude");
        assert_eq!(Family::Copilot.cli(), "copilot");
    }
}
