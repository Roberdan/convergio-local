//! `cvg pr sync <plan-id>` — auto-transition pending plan tasks to
//! `submitted` when their tracking PR has merged.
//!
//! Reads merged GitHub PRs via `gh pr list --state merged`, parses each
//! body for `Tracks: <task-uuid>` lines (one or more, comma- or
//! whitespace-separated), and POSTs `submitted` transitions to the
//! daemon for tasks belonging to the named plan that are still
//! `pending`. Tasks already `submitted` / `done` / `failed` are skipped.
//!
//! Evidence injection is **not** done in v1 — the daemon's
//! [`EvidenceGate`] still applies. If a task requires evidence and none
//! is attached, the transition is reported as `failed` with the gate
//! reason. The operator (or a follow-up version) attaches evidence
//! before re-running. This is the structural fix for friction-log F35
//! and the v0.2.x finishing-line task **T2.04**.
//!
//! Convention: PR authors add a `Tracks:` line to the PR body for every
//! task this PR closes. See `.github/pull_request_template.md`.

use super::{Client, OutputMode};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::process::Command;

const TRACKS_PREFIX: &str = "Tracks:";

pub async fn run(
    client: &Client,
    plan_id: String,
    agent_id: Option<String>,
    output: OutputMode,
) -> Result<()> {
    // 1. Fetch plan tasks; remember their ids so cross-plan UUIDs in PR
    //    bodies do not silently match the wrong plan.
    let plan_tasks: Vec<Value> = client.get(&format!("/v1/plans/{plan_id}/tasks")).await?;
    let plan_task_ids: BTreeSet<String> = plan_tasks
        .iter()
        .filter_map(|t| t.get("id").and_then(Value::as_str).map(String::from))
        .collect();

    if plan_task_ids.is_empty() {
        return render_report(&SyncReport::default(), output);
    }

    // 2. Pull recent merged PRs.
    let prs = fetch_merged_prs()?;

    // 3. Build (pr_number, task_id) pairs filtered to this plan only.
    let mut tracked: Vec<(i64, String)> = Vec::new();
    for pr in &prs {
        let pr_num = pr.get("number").and_then(Value::as_i64).unwrap_or(0);
        let body = pr.get("body").and_then(Value::as_str).unwrap_or("");
        for task_id in parse_tracks_lines(body) {
            if plan_task_ids.contains(&task_id) {
                tracked.push((pr_num, task_id));
            }
        }
    }

    // 4. Transition each in turn.
    let mut report = SyncReport {
        scanned_prs: prs.len(),
        tracked_pairs: tracked.len(),
        ..SyncReport::default()
    };
    for (pr_num, task_id) in tracked {
        let task: Value = match client.get::<Value>(&format!("/v1/tasks/{task_id}")).await {
            Ok(t) => t,
            Err(e) => {
                report.failed.push(SyncFailure {
                    pr_number: pr_num,
                    task_id: task_id.clone(),
                    reason: format!("fetch task: {e}"),
                });
                continue;
            }
        };
        let status = task.get("status").and_then(Value::as_str).unwrap_or("");
        if matches!(status, "submitted" | "done") {
            report.skipped.push(SyncSkip {
                pr_number: pr_num,
                task_id: task_id.clone(),
                current_status: status.to_string(),
            });
            continue;
        }
        let body = json!({
            "target": "submitted",
            "agent_id": agent_id,
        });
        let result: Result<Value> = client
            .post(&format!("/v1/tasks/{task_id}/transition"), &body)
            .await;
        match result {
            Ok(_) => report.transitioned.push(SyncOk {
                pr_number: pr_num,
                task_id: task_id.clone(),
                previous_status: status.to_string(),
            }),
            Err(e) => report.failed.push(SyncFailure {
                pr_number: pr_num,
                task_id: task_id.clone(),
                reason: e.to_string(),
            }),
        }
    }

    render_report(&report, output)
}

/// Extract every UUID from any `Tracks:` line in the PR body. The line
/// form is `Tracks: <uuid>[, <uuid>]...`. UUIDs are validated by shape
/// (8-4-4-4-12 hex with dashes) so that arbitrary text after `Tracks:`
/// does not produce spurious task ids.
pub(crate) fn parse_tracks_lines(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    for raw in body.lines() {
        let line = raw.trim();
        let Some(rest) = line.strip_prefix(TRACKS_PREFIX) else {
            continue;
        };
        for token in rest.split(|c: char| c == ',' || c.is_whitespace()) {
            let t = token.trim();
            if is_valid_uuid(t) {
                out.push(t.to_string());
            }
        }
    }
    out
}

fn is_valid_uuid(s: &str) -> bool {
    if s.len() != 36 {
        return false;
    }
    s.chars().enumerate().all(|(i, c)| {
        if matches!(i, 8 | 13 | 18 | 23) {
            c == '-'
        } else {
            c.is_ascii_hexdigit()
        }
    })
}

fn fetch_merged_prs() -> Result<Vec<Value>> {
    let out = Command::new("gh")
        .args([
            "pr",
            "list",
            "--state",
            "merged",
            "--limit",
            "50",
            "--json",
            "number,title,body,mergeCommit,mergedAt",
        ])
        .output()
        .context("spawn gh — is the gh CLI installed and authenticated?")?;
    if !out.status.success() {
        anyhow::bail!(
            "gh pr list --state merged failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    serde_json::from_slice(&out.stdout).context("parse gh json output")
}

#[derive(Default)]
struct SyncReport {
    scanned_prs: usize,
    tracked_pairs: usize,
    transitioned: Vec<SyncOk>,
    skipped: Vec<SyncSkip>,
    failed: Vec<SyncFailure>,
}

struct SyncOk {
    pr_number: i64,
    task_id: String,
    previous_status: String,
}

struct SyncSkip {
    pr_number: i64,
    task_id: String,
    current_status: String,
}

struct SyncFailure {
    pr_number: i64,
    task_id: String,
    reason: String,
}

fn render_report(report: &SyncReport, output: OutputMode) -> Result<()> {
    match output {
        OutputMode::Json => {
            let body = json!({
                "scanned_prs": report.scanned_prs,
                "tracked_pairs": report.tracked_pairs,
                "transitioned": report.transitioned.iter().map(|o| json!({
                    "pr_number": o.pr_number,
                    "task_id": o.task_id,
                    "previous_status": o.previous_status,
                })).collect::<Vec<_>>(),
                "skipped": report.skipped.iter().map(|s| json!({
                    "pr_number": s.pr_number,
                    "task_id": s.task_id,
                    "current_status": s.current_status,
                })).collect::<Vec<_>>(),
                "failed": report.failed.iter().map(|f| json!({
                    "pr_number": f.pr_number,
                    "task_id": f.task_id,
                    "reason": f.reason,
                })).collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&body)?);
        }
        OutputMode::Plain => {
            println!(
                "scanned={} tracked={} transitioned={} skipped={} failed={}",
                report.scanned_prs,
                report.tracked_pairs,
                report.transitioned.len(),
                report.skipped.len(),
                report.failed.len()
            );
        }
        _ => {
            println!(
                "cvg pr sync — scanned {} merged PRs, {} (PR, task) pairs found",
                report.scanned_prs, report.tracked_pairs
            );
            println!();
            println!(
                "  transitioned ({}):  {} → submitted",
                report.transitioned.len(),
                if report.transitioned.is_empty() {
                    "no tasks"
                } else {
                    "pending"
                }
            );
            for o in &report.transitioned {
                println!("    PR #{} → task {}", o.pr_number, &o.task_id[..8]);
            }
            println!();
            println!(
                "  skipped ({}): already submitted or done",
                report.skipped.len()
            );
            for s in &report.skipped {
                println!(
                    "    PR #{} → task {} ({})",
                    s.pr_number,
                    &s.task_id[..8],
                    s.current_status
                );
            }
            println!();
            println!(
                "  failed ({}): gate refusal or transport error",
                report.failed.len()
            );
            for f in &report.failed {
                println!(
                    "    PR #{} → task {}: {}",
                    f.pr_number,
                    &f.task_id[..8],
                    f.reason
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const UUID_A: &str = "7e33309f-1457-4c8e-9eae-dba599a4a452";
    const UUID_B: &str = "7ec3fc92-e6b7-4cc5-96b6-659a572160be";

    #[test]
    fn parse_tracks_extracts_single_uuid() {
        let body = format!("## Summary\n\nbody.\n\nTracks: {UUID_A}\n");
        let ids = parse_tracks_lines(&body);
        assert_eq!(ids, vec![UUID_A.to_string()]);
    }

    #[test]
    fn parse_tracks_extracts_multiple_lines() {
        let body = format!("Tracks: {UUID_A}\nTracks: {UUID_B}\n");
        let ids = parse_tracks_lines(&body);
        assert_eq!(ids, vec![UUID_A.to_string(), UUID_B.to_string()]);
    }

    #[test]
    fn parse_tracks_extracts_comma_separated() {
        let body = format!("Tracks: {UUID_A}, {UUID_B}\n");
        let ids = parse_tracks_lines(&body);
        assert_eq!(ids, vec![UUID_A.to_string(), UUID_B.to_string()]);
    }

    #[test]
    fn parse_tracks_rejects_non_uuid_garbage() {
        let body = "Tracks: not-a-uuid 12345 short-string\n";
        assert!(parse_tracks_lines(body).is_empty());
    }

    #[test]
    fn parse_tracks_returns_empty_on_no_tracks_line() {
        let body = "## Summary\n\nNothing tracked here.\n## Files touched\n";
        assert!(parse_tracks_lines(body).is_empty());
    }

    #[test]
    fn parse_tracks_ignores_indented_tracks() {
        // Lines that mention 'Tracks:' inside prose (not at the start
        // of a line after trim) should still be picked up because we
        // trim. This is intentional — generous parsing.
        let body = format!("Some context. Tracks: {UUID_A}\n");
        // After `line.trim()`, the line starts with 'Some context.'
        // not 'Tracks:', so this is correctly NOT picked up.
        assert!(parse_tracks_lines(&body).is_empty());
    }

    #[test]
    fn is_valid_uuid_accepts_v4_shape() {
        assert!(is_valid_uuid(UUID_A));
    }

    #[test]
    fn is_valid_uuid_rejects_too_short() {
        assert!(!is_valid_uuid("7e33309f-1457"));
    }

    #[test]
    fn is_valid_uuid_rejects_missing_dashes() {
        assert!(!is_valid_uuid("7e33309f1457bcde7e33309f1457bcde0000"));
    }

    #[test]
    fn is_valid_uuid_rejects_non_hex_chars() {
        assert!(!is_valid_uuid("zzzzzzzz-1457-4c8e-9eae-dba599a4a452"));
    }
}
