//! CLI smoke tests for `cvg capability`.

use assert_cmd::Command;
use predicates::prelude::*;

fn cvg() -> Command {
    Command::cargo_bin("cvg").expect("cvg binary built")
}

#[test]
fn top_level_help_lists_capability() {
    cvg()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("capability"));
}

#[test]
fn capability_help_lists_list() {
    cvg()
        .args(["capability", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"));
}
