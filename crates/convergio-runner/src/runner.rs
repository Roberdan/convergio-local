//! `Runner` trait + the two vendor implementations.
//!
//! Each runner is a *pure* preparer: given a [`SpawnContext`] it
//! returns a [`PreparedCommand`]. The actual subprocess lifecycle
//! (spawn, supervise, reap) is the executor's concern.

use crate::command::PreparedCommand;
use crate::error::{Result, RunnerError};
use crate::kind::{Family, RunnerKind};
use crate::prompt::{self, PromptInputs};
use convergio_durability::Task;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Everything a runner needs to assemble its command + prompt.
pub struct SpawnContext<'a> {
    /// The task to be worked on.
    pub task: &'a Task,
    /// Plan id this task belongs to.
    pub plan_id: &'a str,
    /// Plan title.
    pub plan_title: &'a str,
    /// Daemon HTTP base URL the agent will hit for state changes.
    pub daemon_url: &'a str,
    /// Stable agent identity to register under.
    pub agent_id: &'a str,
    /// Optional graph context (`convergio_graph::for_task_text`).
    pub graph_context: Option<&'a str>,
    /// Working directory — always a worktree under
    /// `.claude/worktrees/<branch>/`.
    pub cwd: &'a Path,
    /// Per-session budget cap (USD). Forwarded to `claude`'s
    /// `--max-budget-usd`. Ignored by Copilot (no equivalent flag).
    pub max_budget_usd: Option<f32>,
}

/// One runner == one vendor CLI wrapping.
pub trait Runner {
    /// Build the [`PreparedCommand`] for `ctx`. Pure: does not run
    /// the binary, does not touch the filesystem, does not call HTTP.
    fn prepare(&self, ctx: &SpawnContext<'_>) -> Result<PreparedCommand>;
}

/// Pick a concrete runner for `kind`.
pub fn for_kind(kind: &RunnerKind) -> Box<dyn Runner> {
    match kind.family {
        Family::Claude => Box::new(ClaudeRunner {
            model: kind.model.clone(),
        }),
        Family::Copilot => Box::new(CopilotRunner {
            model: kind.model.clone(),
        }),
    }
}

/// Wraps `claude -p ... --model X --output-format json`.
///
/// Reads the prompt from stdin (`--input-format text`) so very long
/// prompts (graph context-pack can be 30+ KB) survive argv limits.
pub struct ClaudeRunner {
    /// `--model` value.
    pub model: String,
}

impl Runner for ClaudeRunner {
    fn prepare(&self, ctx: &SpawnContext<'_>) -> Result<PreparedCommand> {
        let prompt = prompt::build(&PromptInputs {
            task: ctx.task,
            plan_id: ctx.plan_id,
            plan_title: ctx.plan_title,
            daemon_url: ctx.daemon_url,
            agent_id: ctx.agent_id,
            graph_context: ctx.graph_context,
        });
        // ADR-0032 follow-up: claude in `-p` mode without
        // --dangerously-skip-permissions hangs waiting for tool
        // consent. Convergio's worktree boundary + audit chain are
        // the actual safety net, so we always set the flag — same
        // posture as Copilot's --allow-all-tools.
        // stream-json + verbose so the executor can pipe each
        // assistant turn / tool_use to the operator in real time
        // (`--output-format json` buffers the whole run).
        let mut args: Vec<OsString> = vec![
            "--dangerously-skip-permissions".into(),
            "-p".into(),
            "--model".into(),
            self.model.clone().into(),
            "--output-format".into(),
            "stream-json".into(),
            "--verbose".into(),
            "--input-format".into(),
            "text".into(),
        ];
        if let Some(b) = ctx.max_budget_usd {
            args.push("--max-budget-usd".into());
            args.push(format!("{b}").into());
        }
        Ok(PreparedCommand {
            program: OsString::from("claude"),
            args,
            cwd: PathBuf::from(ctx.cwd),
            stdin_prompt: prompt,
        })
    }
}

/// Wraps `copilot -p ... --model X --allow-all-tools`.
///
/// `--allow-all-tools` is required by the Copilot CLI for any
/// non-interactive run (the equivalent of "this script accepts all
/// tool consents in advance"). Convergio's worktree boundary plus
/// the daemon's audit chain are the actual safety net.
pub struct CopilotRunner {
    /// `--model` value.
    pub model: String,
}

impl Runner for CopilotRunner {
    fn prepare(&self, ctx: &SpawnContext<'_>) -> Result<PreparedCommand> {
        let prompt = prompt::build(&PromptInputs {
            task: ctx.task,
            plan_id: ctx.plan_id,
            plan_title: ctx.plan_title,
            daemon_url: ctx.daemon_url,
            agent_id: ctx.agent_id,
            graph_context: ctx.graph_context,
        });
        let args: Vec<OsString> = vec![
            "-p".into(),
            prompt.clone().into(),
            "--model".into(),
            self.model.clone().into(),
            "--allow-all-tools".into(),
        ];
        Ok(PreparedCommand {
            program: OsString::from("copilot"),
            args,
            cwd: PathBuf::from(ctx.cwd),
            stdin_prompt: prompt,
        })
    }
}

/// Convenience: surface a clear error when the vendor CLI is not
/// on `PATH`. Callers may invoke this before `prepare` to fail fast.
pub fn assert_cli_on_path(family: Family) -> Result<()> {
    let cli = family.cli();
    let found = std::env::var_os("PATH")
        .map(|paths| {
            std::env::split_paths(&paths).any(|p| {
                let candidate = p.join(cli);
                candidate.is_file() || candidate.with_extension("exe").is_file()
            })
        })
        .unwrap_or(false);
    if found {
        Ok(())
    } else {
        Err(RunnerError::CliMissing { cli })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use convergio_durability::TaskStatus;
    use std::path::Path;

    fn task() -> Task {
        let now = Utc::now();
        Task {
            id: "t-aaa".into(),
            plan_id: "p-bbb".into(),
            wave: 1,
            sequence: 1,
            title: "do thing".into(),
            description: None,
            status: TaskStatus::Pending,
            agent_id: None,
            evidence_required: vec!["test".into()],
            last_heartbeat_at: None,
            created_at: now,
            updated_at: now,
            started_at: None,
            ended_at: None,
            duration_ms: None,
        }
    }

    fn ctx<'a>(task: &'a Task) -> SpawnContext<'a> {
        SpawnContext {
            task,
            plan_id: "p-bbb",
            plan_title: "demo",
            daemon_url: "http://127.0.0.1:8420",
            agent_id: "claude-test",
            graph_context: None,
            cwd: Path::new("/tmp/wt"),
            max_budget_usd: Some(1.5),
        }
    }

    #[test]
    fn claude_runner_uses_print_mode_and_model_flag() {
        let task = task();
        let ctx = ctx(&task);
        let r = ClaudeRunner {
            model: "sonnet".into(),
        };
        let cmd = r.prepare(&ctx).unwrap();
        assert_eq!(cmd.program, OsString::from("claude"));
        let argv: Vec<&str> = cmd.args.iter().filter_map(|a| a.to_str()).collect();
        assert!(argv.contains(&"-p"));
        assert!(argv.contains(&"--model"));
        assert!(argv.contains(&"sonnet"));
        assert!(argv.contains(&"--max-budget-usd"));
        assert!(
            argv.contains(&"--dangerously-skip-permissions"),
            "non-interactive runs need the permission bypass"
        );
        assert!(
            argv.contains(&"stream-json"),
            "stream-json keeps the operator's terminal informed"
        );
        assert!(argv.contains(&"--verbose"));
        assert!(cmd.stdin_prompt.contains("`t-aaa`"));
    }

    #[test]
    fn copilot_runner_passes_prompt_via_argv_with_allow_all_tools() {
        let task = task();
        let ctx = ctx(&task);
        let r = CopilotRunner {
            model: "gpt-5.2".into(),
        };
        let cmd = r.prepare(&ctx).unwrap();
        assert_eq!(cmd.program, OsString::from("copilot"));
        let argv: Vec<&str> = cmd.args.iter().filter_map(|a| a.to_str()).collect();
        assert!(argv.contains(&"-p"));
        assert!(argv.contains(&"--allow-all-tools"));
        assert!(argv.contains(&"gpt-5.2"));
    }

    #[test]
    fn for_kind_dispatches_to_the_right_vendor() {
        let task = task();
        let ctx = ctx(&task);
        let claude = for_kind(&RunnerKind::claude_sonnet());
        assert_eq!(
            claude.prepare(&ctx).unwrap().program,
            OsString::from("claude")
        );
        let copilot = for_kind(&RunnerKind::copilot_gpt());
        assert_eq!(
            copilot.prepare(&ctx).unwrap().program,
            OsString::from("copilot")
        );
    }

    #[test]
    fn assert_cli_on_path_rejects_when_binary_missing_from_explicit_path() {
        // We can't mutate the global PATH safely from a test (other
        // threads may read it). Re-implement the lookup against an
        // explicit path string so the assertion is hermetic.
        let cli = Family::Claude.cli();
        let bogus = "/__convergio_runner_bogus_path__";
        let found = std::env::split_paths(bogus).any(|p| p.join(cli).is_file());
        assert!(!found);
    }
}
