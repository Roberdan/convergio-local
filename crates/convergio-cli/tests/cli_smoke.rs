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
        .stdout(predicate::str::contains("setup"))
        .stdout(predicate::str::contains("doctor"))
        .stdout(predicate::str::contains("plan"))
        .stdout(predicate::str::contains("task"))
        .stdout(predicate::str::contains("evidence"))
        .stdout(predicate::str::contains("demo"))
        .stdout(predicate::str::contains("audit"));
}

#[test]
fn setup_help_lists_init() {
    cvg()
        .args(["setup", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("init"));
}

#[test]
fn doctor_help_lists_json() {
    cvg()
        .args(["doctor", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--json"));
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
fn task_help_lists_subcommands() {
    cvg()
        .args(["task", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("transition"));
}

#[test]
fn evidence_help_lists_subcommands() {
    cvg()
        .args(["evidence", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("add"))
        .stdout(predicate::str::contains("list"));
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
fn doctor_json_reports_unreachable_daemon() {
    cvg()
        .args(["--url", "http://127.0.0.1:1", "doctor", "--json"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"ok\": false"))
        .stdout(predicate::str::contains("\"name\": \"daemon\""));
}

#[test]
fn setup_creates_config_under_home() {
    let home = tempfile::tempdir().expect("temp home");
    cvg()
        .env("HOME", home.path())
        .arg("setup")
        .assert()
        .success()
        .stdout(predicate::str::contains("Setup complete"));
    assert!(home.path().join(".convergio/config.toml").is_file());
    assert!(home.path().join(".convergio/adapters").is_dir());
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
