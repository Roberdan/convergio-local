//! CLI smoke tests for `cvg setup agent <host>` — split from
//! `cli_smoke.rs` to keep both files under the 300-line cap
//! (CONSTITUTION § 13).
//!
//! Tests here verify the host-specific output of `cvg setup agent`,
//! particularly the Claude Code extras shipped by Wave 0b
//! (.claude/settings.json hook template + cvg-attach skill bundle).

use assert_cmd::Command;
use predicates::prelude::*;

fn cvg() -> Command {
    Command::cargo_bin("cvg").expect("cvg binary built")
}

#[test]
fn setup_agent_claude_emits_skill_and_settings() {
    let home = tempfile::tempdir().expect("temp home");
    cvg()
        .env("HOME", home.path())
        .args(["setup", "agent", "claude"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Claude Code extras"));
    let dir = home.path().join(".convergio/adapters/claude");
    assert!(dir.join("settings.json").is_file());
    assert!(dir.join("skill-cvg-attach/SKILL.md").is_file());
    assert!(dir.join("skill-cvg-attach/cvg-attach.sh").is_file());
    let settings = std::fs::read_to_string(dir.join("settings.json")).unwrap();
    assert!(settings.contains("SessionStart"));
    assert!(settings.contains("cvg-attach.sh"));
}

#[test]
fn setup_agent_copilot_does_not_emit_claude_extras() {
    let home = tempfile::tempdir().expect("temp home");
    cvg()
        .env("HOME", home.path())
        .args(["setup", "agent", "copilot-local"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Adapter snippets created")
                .and(predicate::str::contains("Claude Code extras").not()),
        );
    let dir = home.path().join(".convergio/adapters/copilot-local");
    assert!(!dir.join("settings.json").exists());
    assert!(!dir.join("skill-cvg-attach").exists());
}
