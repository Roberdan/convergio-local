//! `gh pr list` shell-out helpers.
//!
//! Split out from [`crate::client`] so the HTTP fetcher stays focused
//! and so the 300-line cap is respected. The gh integration is
//! intentionally simple: one subprocess invocation per refresh, no
//! caching beyond what the dashboard already does at tick granularity.

use crate::client::PrSummary;
use anyhow::Result;
use std::process::Command;

/// Run `gh pr list` and parse the JSON. Returns an empty vec on any
/// error so the dashboard can render the rest of the snapshot.
pub fn fetch_open_prs() -> Result<Vec<PrSummary>> {
    let out = Command::new("gh")
        .args([
            "pr",
            "list",
            "--state",
            "open",
            "--json",
            "number,title,headRefName,statusCheckRollup",
        ])
        .output();
    let out = match out {
        Ok(o) if o.status.success() => o,
        _ => return Ok(Vec::new()),
    };
    let raw: Vec<serde_json::Value> = serde_json::from_slice(&out.stdout)?;
    let mut prs = Vec::with_capacity(raw.len());
    for v in raw {
        let ci = ci_rollup(&v);
        let pr: PrSummary = serde_json::from_value(serde_json::json!({
            "number": v.get("number").cloned().unwrap_or(serde_json::json!(0)),
            "title": v.get("title").cloned().unwrap_or_default(),
            "headRefName": v.get("headRefName").cloned().unwrap_or_default(),
            "ci": ci,
        }))?;
        prs.push(pr);
    }
    Ok(prs)
}

/// Roll a PR's `statusCheckRollup` into one of `success`, `failure`,
/// `pending`, or empty string.
pub fn ci_rollup(v: &serde_json::Value) -> String {
    let checks = match v.get("statusCheckRollup").and_then(|c| c.as_array()) {
        Some(c) => c,
        None => return String::new(),
    };
    if checks.is_empty() {
        return String::new();
    }
    let mut any_failure = false;
    let mut any_pending = false;
    for c in checks {
        let conclusion = c
            .get("conclusion")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let status = c
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if conclusion == "failure" || conclusion == "cancelled" {
            any_failure = true;
        }
        if status != "completed" {
            any_pending = true;
        }
    }
    if any_failure {
        "failure".into()
    } else if any_pending {
        "pending".into()
    } else {
        "success".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ci_rollup_classifies_failure_first() {
        let v = serde_json::json!({"statusCheckRollup": [
            {"status": "completed", "conclusion": "success"},
            {"status": "completed", "conclusion": "failure"},
        ]});
        assert_eq!(ci_rollup(&v), "failure");
    }

    #[test]
    fn ci_rollup_pending_when_any_in_progress() {
        let v = serde_json::json!({"statusCheckRollup": [
            {"status": "completed", "conclusion": "success"},
            {"status": "in_progress", "conclusion": ""},
        ]});
        assert_eq!(ci_rollup(&v), "pending");
    }

    #[test]
    fn ci_rollup_success_when_all_clean() {
        let v = serde_json::json!({"statusCheckRollup": [
            {"status": "completed", "conclusion": "success"},
            {"status": "completed", "conclusion": "success"},
        ]});
        assert_eq!(ci_rollup(&v), "success");
    }

    #[test]
    fn ci_rollup_empty_when_no_checks() {
        let v = serde_json::json!({});
        assert_eq!(ci_rollup(&v), "");
        let v = serde_json::json!({"statusCheckRollup": []});
        assert_eq!(ci_rollup(&v), "");
    }
}
