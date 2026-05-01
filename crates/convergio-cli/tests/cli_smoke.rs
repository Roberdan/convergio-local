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
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("setup"))
        .stdout(predicate::str::contains("doctor"))
        .stdout(predicate::str::contains("plan"))
        .stdout(predicate::str::contains("task"))
        .stdout(predicate::str::contains("evidence"))
        .stdout(predicate::str::contains("crdt"))
        .stdout(predicate::str::contains("workspace"))
        .stdout(predicate::str::contains("mcp"))
        .stdout(predicate::str::contains("service"))
        .stdout(predicate::str::contains("demo"))
        .stdout(predicate::str::contains("audit"));
}

#[test]
fn setup_help_lists_init() {
    cvg()
        .args(["setup", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("agent"));
}

#[test]
fn doctor_help_lists_json() {
    cvg()
        .args(["doctor", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--json"));
}

// status help tests moved to `cli_smoke_status.rs` to keep this
// file under the 300-line cap (CONSTITUTION § 13).

#[test]
fn status_help_lists_agents_flag() {
    cvg()
        .args(["status", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--agents"));
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
fn global_output_json_works_for_health() {
    cvg()
        .args(["--url", "http://127.0.0.1:1", "--output", "json", "health"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Could not reach daemon"));
}

#[test]
fn doctor_accepts_global_plain_output() {
    cvg()
        .args(["--url", "http://127.0.0.1:1", "--output", "plain", "doctor"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("fail"));
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
fn plan_create_help_lists_project() {
    cvg()
        .args(["plan", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--project"));
}

#[test]
fn plan_create_accepts_global_output_modes() {
    let url = "http://127.0.0.1:1";
    for mode in ["human", "json", "plain"] {
        let args = ["--url", url, "--output", mode, "plan", "create", "x"];
        cvg().args(args).assert().failure();
    }
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
fn crdt_help_lists_conflicts() {
    cvg()
        .args(["crdt", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("conflicts"));
}

#[test]
fn workspace_help_lists_leases() {
    cvg()
        .args(["workspace", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("leases"));
}

#[test]
fn mcp_help_lists_tail() {
    cvg()
        .args(["mcp", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tail"));
}

#[test]
fn mcp_tail_without_log_is_clear() {
    let home = tempfile::tempdir().expect("temp home");
    cvg()
        .env("HOME", home.path())
        .args(["mcp", "tail"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No MCP log"));
}

#[test]
fn service_help_lists_subcommands() {
    cvg()
        .args(["service", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("install"))
        .stdout(predicate::str::contains("start"))
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("uninstall"));
}

#[test]
fn task_help_lists_subcommands() {
    cvg()
        .args(["task", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("transition"))
        .stdout(predicate::str::contains("heartbeat"));
}

// task create coverage lives in `crates/convergio-cli/tests/cli_task_create.rs`.

// ADR-0011 CLI regressions live in
// `crates/convergio-cli/tests/cli_thor_only_done.rs`.

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
fn doctor_json_with_stale_pid_keeps_stderr_clean() {
    let home = tempfile::tempdir().expect("temp home");
    let config_dir = home.path().join(".convergio");
    std::fs::create_dir_all(&config_dir).expect("config dir");
    std::fs::write(config_dir.join("daemon.pid"), "999999").expect("pid file");
    cvg()
        .env("HOME", home.path())
        .args(["--url", "http://127.0.0.1:1", "doctor", "--json"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("stale pid"))
        .stderr(
            predicate::str::contains("doctor found failing checks")
                .and(predicate::str::contains("kill:").not()),
        );
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
fn setup_agent_creates_snippets_under_home() {
    let home = tempfile::tempdir().expect("temp home");
    cvg()
        .env("HOME", home.path())
        .args(["setup", "agent", "claude"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Adapter snippets created"));
    let dir = home.path().join(".convergio/adapters/claude");
    assert!(dir.join("mcp.json").is_file());
    assert!(dir.join("prompt.txt").is_file());
    assert!(dir.join("README.txt").is_file());
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
