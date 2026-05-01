//! Filesystem scanner for [`super::WireCheckGate`].
//!
//! Two surfaces:
//!
//! 1. **Route check** — concatenate the text of every `*.rs` file
//!    under `crates/convergio-server/src/routes/` once, then
//!    substring-match `.route("PATH"` for each claim.
//! 2. **CLI check** — for a claim like `"<top> <sub>"`, confirm
//!    that `crates/convergio-cli/src/commands/<top>.rs` exists and
//!    contains the `<sub>` token (case-insensitive).
//!
//! Kept in a sibling module so the orchestrator file
//! (`wire_check_gate.rs`) stays under the 300-line cap and the
//! filesystem-poking surface is testable in isolation.

use std::fs;
use std::path::{Path, PathBuf};

/// Read every `*.rs` file under `routes_root` and return the
/// concatenation. On any I/O error, returns the empty string —
/// downstream the gate will report every claimed route as missing,
/// which is the right outcome (the agent's claim cannot be
/// verified).
pub(super) fn collect_route_text(routes_root: &Path) -> String {
    let mut out = String::new();
    let mut stack: Vec<PathBuf> = vec![routes_root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            if let Ok(text) = fs::read_to_string(&path) {
                out.push_str(&text);
                out.push('\n');
            }
        }
    }
    out
}

/// True iff `haystack` contains a `.route("<path>"` literal for
/// the given path. We accept the literal in any whitespace shape
/// that `cargo fmt` produces — leading whitespace inside the
/// parentheses is normalised to a single space, so a substring
/// match on `.route("` plus the exact path is sound.
pub(super) fn route_is_mounted(haystack: &str, path: &str) -> bool {
    let needle = format!(".route(\"{path}\"");
    haystack.contains(&needle)
}

/// True iff `cli` (e.g. `"plan list"`) maps to an existing top-level
/// CLI module that mentions the subcommand token.
///
/// Heuristic — see [`super`] module docs for what this does NOT catch.
pub(super) fn cli_path_exists(commands_root: &Path, cli: &str) -> bool {
    let mut parts = cli.split_whitespace();
    let top = match parts.next() {
        Some(t) if !t.is_empty() => t.to_ascii_lowercase(),
        _ => return false,
    };
    let module = commands_root.join(format!("{top}.rs"));
    let text = match fs::read_to_string(&module) {
        Ok(t) => t,
        Err(_) => return false,
    };
    let lower = text.to_ascii_lowercase();
    // No subcommand component → existence of the top-level module is
    // the entire claim, and we have just confirmed it.
    let mut ok = true;
    for sub in parts {
        if sub.is_empty() {
            continue;
        }
        if !lower.contains(&sub.to_ascii_lowercase()) {
            ok = false;
            break;
        }
    }
    ok
}
