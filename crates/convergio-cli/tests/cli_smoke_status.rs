//! CLI smoke tests for `cvg status` — split from `cli_smoke.rs` to
//! keep both files under the 300-line cap (CONSTITUTION § 13).

use assert_cmd::Command;
use predicates::prelude::*;

fn cvg() -> Command {
    Command::cargo_bin("cvg").expect("cvg binary built")
}

#[test]
fn status_help_lists_completed_limit() {
    cvg()
        .args(["status", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--completed-limit"));
}

#[test]
fn status_help_lists_v2_flags() {
    cvg()
        .args(["status", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--show-waves"))
        .stdout(predicate::str::contains("--mine"));
}
