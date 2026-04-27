//! CLI smoke tests — exercise the binary directly so we catch
//! regressions in the clap definitions / wiring without booting a
//! daemon.

use assert_cmd::Command;
use predicates::prelude::*;

fn cvg() -> Command {
    Command::cargo_bin("cvg").expect("cvg binary built")
}

#[test]
fn help_lists_known_subcommands() {
    cvg()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("health"))
        .stdout(predicate::str::contains("plan"))
        .stdout(predicate::str::contains("audit"));
}

#[test]
fn version_reports_cargo_pkg_version() {
    let expected = env!("CARGO_PKG_VERSION");
    cvg()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(expected));
}

#[test]
fn plan_help_lists_subcommands() {
    cvg()
        .args(["plan", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"));
}

#[test]
fn audit_help_lists_verify() {
    cvg()
        .args(["audit", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("verify"));
}

#[test]
fn unknown_subcommand_fails_with_error() {
    cvg()
        .arg("nope")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized").or(predicate::str::contains("invalid")));
}

#[test]
fn health_against_unreachable_url_fails_clearly_in_english() {
    // Bind to a port nothing is listening on. Force English so the
    // localized message is predictable regardless of CI's LANG.
    cvg()
        .args(["--lang", "en", "--url", "http://127.0.0.1:1", "health"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Could not reach daemon"));
}

#[test]
fn health_against_unreachable_url_localizes_to_italian() {
    cvg()
        .args(["--lang", "it", "--url", "http://127.0.0.1:1", "health"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Impossibile raggiungere"));
}

#[test]
fn lang_flag_is_global_under_subcommands() {
    // `--lang` must work positioned after the subcommand too.
    cvg()
        .args(["health", "--lang", "it", "--url", "http://127.0.0.1:1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Impossibile raggiungere"));
}
