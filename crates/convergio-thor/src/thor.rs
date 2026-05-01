//! `Thor::validate` — produce a verdict for a plan.
//!
//! v0 only checked evidence shape: every task must be `submitted` or
//! `done`, and every required evidence kind must be attached. Smart
//! Thor (T3.02, ADR-0012 implementation slice) adds a third gate:
//! before promoting `submitted -> done`, run the project's actual
//! pipeline (test suite, build, custom checks) and refuse if it
//! fails. The pipeline command is configured via the
//! `CONVERGIO_THOR_PIPELINE_CMD` environment variable; when unset,
//! Thor falls back to the v0 evidence-only behaviour for backwards
//! compatibility.

use crate::error::Result;
use convergio_durability::{Durability, TaskStatus};
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Validator verdict.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "verdict")]
pub enum Verdict {
    /// Plan passes — safe to close.
    Pass,
    /// Plan fails — list of reasons (one per failing task / missing
    /// evidence kind / failed pipeline).
    Fail {
        /// Why the plan was rejected.
        reasons: Vec<String>,
    },
}

/// Environment variable that, when set, makes Thor run the named
/// shell command before promoting submitted tasks to done.
pub const PIPELINE_ENV: &str = "CONVERGIO_THOR_PIPELINE_CMD";

/// Thor validator handle.
#[derive(Clone)]
pub struct Thor {
    durability: Durability,
    pipeline_cmd: Option<String>,
}

impl Thor {
    /// Wrap a [`Durability`] facade. Reads the optional pipeline
    /// command from `CONVERGIO_THOR_PIPELINE_CMD` (T3.02). When the
    /// variable is unset or empty, Thor behaves like v0 — pure
    /// evidence-shape validation.
    pub fn new(durability: Durability) -> Self {
        let pipeline_cmd = std::env::var(PIPELINE_ENV)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        Self {
            durability,
            pipeline_cmd,
        }
    }

    /// Build with an explicit pipeline command, ignoring the
    /// environment. Useful for tests and for embedding Thor inside
    /// a daemon that wants its own configuration source.
    pub fn with_pipeline(durability: Durability, pipeline_cmd: Option<String>) -> Self {
        Self {
            durability,
            pipeline_cmd: pipeline_cmd.filter(|s| !s.is_empty()),
        }
    }

    /// Validate every task of `plan_id`. Equivalent to
    /// [`Self::validate_wave`] called with `wave = None`.
    ///
    /// Plan-strict: even one `pending`/`failed` task in any wave
    /// fails the verdict. For wave-scoped validation (T3.06 — close
    /// the OODA loop on long-running backlog plans without first
    /// shipping every pending task) use [`Self::validate_wave`].
    pub async fn validate(&self, plan_id: &str) -> Result<Verdict> {
        self.validate_wave(plan_id, None).await
    }

    /// Validate the tasks of `plan_id`, optionally restricted to a
    /// single `wave` number (T3.06).
    ///
    /// A task is valid when:
    ///
    /// 1. its status is `submitted` or `done` (anything else fails);
    /// 2. every kind listed in `evidence_required` has at least one
    ///    matching evidence row.
    ///
    /// On a passing verdict, every task currently in `submitted` is
    /// promoted to `done` atomically through
    /// [`Durability::complete_validated_tasks`]. This is the **only**
    /// path that sets `done` (CONSTITUTION §6, ADR-0011) — agents may
    /// never self-promote via `transition_task`.
    ///
    /// When `wave` is `Some(N)`, only tasks with `wave == N` are
    /// considered. Tasks in other waves are ignored: they neither
    /// block the verdict nor get promoted by it. This lets backlog
    /// plans (v0.1.x, v0.2, v0.3) close the OODA loop wave by wave
    /// instead of having to swallow the whole pending backlog at
    /// once.
    ///
    /// The verdict is idempotent: validating a plan whose tasks are
    /// already all `done` simply returns `Pass` with zero promotions.
    pub async fn validate_wave(&self, plan_id: &str, wave: Option<i64>) -> Result<Verdict> {
        // Confirm the plan exists — yields NotFound otherwise.
        self.durability.plans().get(plan_id).await?;

        let all_tasks = self.durability.tasks().list_by_plan(plan_id).await?;
        let tasks: Vec<_> = match wave {
            Some(w) => all_tasks.into_iter().filter(|t| t.wave == w).collect(),
            None => all_tasks,
        };
        if tasks.is_empty() {
            let reason = match wave {
                Some(w) => format!("plan has no tasks in wave {w}"),
                None => "plan has no tasks".into(),
            };
            return Ok(Verdict::Fail {
                reasons: vec![reason],
            });
        }

        let mut reasons = Vec::new();
        let mut to_promote: Vec<String> = Vec::new();
        for task in tasks {
            match task.status {
                TaskStatus::Submitted | TaskStatus::Done => {}
                TaskStatus::Failed => {
                    reasons.push(format!(
                        "task {} ({}) is failed — plan cannot validate",
                        task.id, task.title
                    ));
                    continue;
                }
                _ => {
                    reasons.push(format!(
                        "task {} ({}) is {} — expected submitted or done",
                        task.id,
                        task.title,
                        task.status.as_str()
                    ));
                    continue;
                }
            }
            let kinds = self.durability.evidence().kinds_for(&task.id).await?;
            let mut task_ok = true;
            for required in &task.evidence_required {
                if !kinds.iter().any(|k| k == required) {
                    reasons.push(format!(
                        "task {} ({}) missing evidence kind '{}'",
                        task.id, task.title, required
                    ));
                    task_ok = false;
                }
            }
            if task_ok && matches!(task.status, TaskStatus::Submitted) {
                to_promote.push(task.id.clone());
            }
        }

        if !reasons.is_empty() {
            return Ok(Verdict::Fail { reasons });
        }

        // T3.02 / ADR-0012: smart Thor runs the project's pipeline
        // before promoting. Pipeline failure reuses the same
        // `Verdict::Fail` shape so callers do not need a third
        // status. The pipeline tail (last 4 KiB of merged stdout +
        // stderr) goes into the reason so the agent can see what
        // broke without re-running it.
        if let Some(cmd) = &self.pipeline_cmd {
            if let Some(reason) = run_pipeline(cmd) {
                return Ok(Verdict::Fail {
                    reasons: vec![reason],
                });
            }
        }

        // Pass: promote every still-submitted task to done atomically.
        // Empty list is a no-op (idempotent re-validate).
        self.durability
            .complete_validated_tasks(&to_promote)
            .await?;
        Ok(Verdict::Pass)
    }
}

/// Run `cmd` via `sh -c`. Returns `None` on success, `Some(reason)`
/// on failure (the reason is suitable for `Verdict::Fail::reasons`).
///
/// We use `sh -c` so the user can pass pipelines and env-var
/// expansion in one string (`cargo test --workspace 2>&1 | tail -20`).
/// The 4 KiB tail keeps the verdict payload bounded — long test
/// outputs would otherwise drown the audit log.
fn run_pipeline(cmd: &str) -> Option<String> {
    let out = Command::new("sh").arg("-c").arg(cmd).output();
    match out {
        Ok(o) if o.status.success() => None,
        Ok(o) => {
            let mut tail = String::from_utf8_lossy(&o.stdout).into_owned();
            tail.push_str(&String::from_utf8_lossy(&o.stderr));
            const TAIL_BYTES: usize = 4096;
            let trimmed = if tail.len() > TAIL_BYTES {
                &tail[tail.len() - TAIL_BYTES..]
            } else {
                tail.as_str()
            };
            Some(format!(
                "pipeline `{cmd}` failed (exit={}): {trimmed}",
                o.status.code().unwrap_or(-1)
            ))
        }
        Err(e) => Some(format!("pipeline `{cmd}` could not be invoked: {e}")),
    }
}
