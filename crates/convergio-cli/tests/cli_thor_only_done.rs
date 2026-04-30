//! ADR-0011 CLI regressions: `cvg task transition` may not target
//! `done`. The clap value enum must omit it so the command fails to
//! parse before any HTTP round-trip.

use assert_cmd::Command;
use predicates::prelude::*;

fn cvg() -> Command {
    Command::cargo_bin("cvg").expect("cvg binary built")
}

#[test]
fn task_transition_target_done_is_rejected_at_clap() {
    cvg()
        .args(["task", "transition", "task-id-placeholder", "done"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value 'done'"));
}

#[test]
fn task_transition_help_lists_only_agent_settable_targets() {
    cvg()
        .args(["task", "transition", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("in-progress"))
        .stdout(predicate::str::contains("submitted"))
        .stdout(predicate::str::contains("failed"))
        .stdout(predicate::str::contains("pending"))
        // `done` is intentionally absent — set only by `cvg validate`.
        .stdout(predicate::str::contains("done").not());
}
