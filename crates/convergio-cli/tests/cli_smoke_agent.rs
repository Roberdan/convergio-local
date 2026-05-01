//! CLI smoke tests for `cvg agent` — split from `cli_smoke.rs` to
//! keep both files under the 300-line cap (CONSTITUTION § 13).

use assert_cmd::Command;
use predicates::prelude::*;

fn cvg() -> Command {
    Command::cargo_bin("cvg").expect("cvg binary built")
}

#[test]
fn agent_help_lists_subcommands() {
    cvg()
        .args(["agent", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("show"));
}

#[test]
fn agent_list_against_unreachable_url_fails_clearly() {
    cvg()
        .args(["--url", "http://127.0.0.1:1", "agent", "list"])
        .assert()
        .failure();
}

#[test]
fn agent_show_help_lists_id_arg() {
    cvg()
        .args(["agent", "show", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<ID>"));
}
