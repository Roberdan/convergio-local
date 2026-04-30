//! T0 regression: `cvg task create` exposes the rich-task surface
//! the daemon already supports (title, description, wave, sequence,
//! evidence_required) without forcing callers to fall back to raw
//! `curl POST /v1/plans/:id/tasks` calls.

use assert_cmd::Command;
use predicates::prelude::*;

fn cvg() -> Command {
    Command::cargo_bin("cvg").expect("cvg binary built")
}

#[test]
fn task_create_help_lists_rich_flags() {
    cvg()
        .args(["task", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<PLAN_ID>"))
        .stdout(predicate::str::contains("<TITLE>"))
        .stdout(predicate::str::contains("--description"))
        .stdout(predicate::str::contains("--wave"))
        .stdout(predicate::str::contains("--sequence"))
        .stdout(predicate::str::contains("--evidence-required"));
}

#[test]
fn task_create_against_unreachable_daemon_fails_clearly() {
    cvg()
        .args([
            "--url",
            "http://127.0.0.1:1",
            "task",
            "create",
            "plan-id-placeholder",
            "title-placeholder",
        ])
        .assert()
        .failure();
}
