//! `NoDebtGate` — refuses `submitted`/`done` transitions when the
//! evidence payload contains technical-debt markers (TODO, FIXME,
//! unwrap, ignored tests, debug prints, ...).
//!
//! This is one of the gates that turns Convergio into a leash for
//! agents: an LLM that has been told "no TODOs" but tries to slip one
//! in *will* get a 409 here, because the evidence it itself attached
//! contains the marker. Either it cleans up, or the task does not
//! advance.
//!
//! ## Defaults
//!
//! See [`default_rules`] for the built-in rule set. New defaults must
//! ship with both a unit test and an ADR justifying the addition —
//! adding rules to a default-on gate is a breaking change for users.
//!
//! ## Customization
//!
//! Build with [`NoDebtGate::with_rules`] to override the rule set
//! (load from `convergio-debt-rules.toml`, etc). The MVP ships
//! defaults only; custom config is a future-session feature.

use super::{Gate, GateContext};
use crate::error::{DurabilityError, Result};
use crate::model::TaskStatus;
use crate::store::EvidenceStore;
use regex::Regex;

/// Refuses the transition when the task's evidence contains debt
/// markers.
pub struct NoDebtGate {
    rules: Vec<DebtRule>,
}

/// One pattern that, if matched anywhere in any evidence payload,
/// blocks the transition.
pub struct DebtRule {
    /// Stable name (used in error messages).
    pub name: &'static str,
    /// Compiled regex.
    pub pattern: Regex,
}

impl Default for NoDebtGate {
    fn default() -> Self {
        Self {
            rules: default_rules(),
        }
    }
}

impl NoDebtGate {
    /// Build with a custom rule set.
    pub fn with_rules(rules: Vec<DebtRule>) -> Self {
        Self { rules }
    }
}

/// Built-in rule set. Each entry is `(name, regex)`.
///
/// Conservative on purpose — false positives here mean a developer
/// can't ship. We only include patterns that are universally smelly:
///
/// - `TODO|FIXME|XXX|HACK` — explicit "I'm leaving debt" markers
/// - `\.unwrap\(\)` / `\.expect\(` — Rust panic-on-error shortcuts
/// - `panic!\(` / `unimplemented!\(` / `todo!\(` — explicit panics
/// - `#\[ignore\]` — disabled tests pretending to pass
/// - `\bdbg!\(` — Rust debug print left in
/// - `console\.log\(` — JS debug print left in
fn default_rules() -> Vec<DebtRule> {
    let entries: &[(&'static str, &'static str)] = &[
        ("todo_marker", r"(?i)\b(TODO|FIXME|XXX|HACK)\b"),
        ("rust_unwrap", r"\.unwrap\(\)"),
        ("rust_expect", r"\.expect\("),
        ("rust_panic", r"\bpanic!\("),
        ("rust_unimplemented", r"\bunimplemented!\("),
        ("rust_todo_macro", r"\btodo!\("),
        ("rust_ignored_test", r"#\[ignore\]"),
        ("rust_dbg", r"\bdbg!\("),
        ("js_console_log", r"console\.log\("),
    ];
    entries
        .iter()
        .filter_map(|(name, pat)| {
            Regex::new(pat).ok().map(|r| DebtRule {
                name,
                pattern: r,
            })
        })
        .collect()
}

#[async_trait::async_trait]
impl Gate for NoDebtGate {
    fn name(&self) -> &'static str {
        "no_debt"
    }

    async fn check(&self, ctx: &GateContext) -> Result<()> {
        if !matches!(
            ctx.target_status,
            TaskStatus::Submitted | TaskStatus::Done
        ) {
            return Ok(());
        }

        let store = EvidenceStore::new(ctx.pool.clone());
        let evidence = store.list_by_task(&ctx.task.id).await?;

        let mut violations: Vec<String> = Vec::new();
        for ev in evidence {
            // Serialize the JSON payload to a single string. Any debt
            // marker in any nested field will match.
            let blob = ev.payload.to_string();
            for rule in &self.rules {
                if rule.pattern.is_match(&blob) {
                    violations.push(format!("{}#{}", ev.kind, rule.name));
                }
            }
        }

        if violations.is_empty() {
            Ok(())
        } else {
            violations.sort();
            violations.dedup();
            Err(DurabilityError::GateRefused {
                gate: "no_debt",
                reason: format!("debt markers found in evidence: {}", violations.join(", ")),
            })
        }
    }
}
