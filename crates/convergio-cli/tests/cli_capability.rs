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

#[test]
fn capability_help_lists_install_file() {
    cvg()
        .args(["capability", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("install-file"));
}

#[test]
fn capability_help_lists_verify_signature() {
    cvg()
        .args(["capability", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("verify-signature"));
}

#[test]
fn capability_help_lists_disable_and_remove() {
    cvg()
        .args(["capability", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("disable"))
        .stdout(predicate::str::contains("remove"));
}
