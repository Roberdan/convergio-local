//! Human renderer for `cvg status` (split out per the
//! `<cmd>.rs` + `<cmd>_render.rs` pattern used by `session`/`pr`).

use convergio_i18n::Bundle;

use super::status::{CompletedTask, PlanSummary, StatusResponse, TaskCounts, TaskSummary};

/// Maximum visible width of a plan description in the human view.
/// Full description remains accessible via `--output json`.
const DESCRIPTION_TRIM: usize = 80;

/// Width (in cells) of the ASCII progress bar.
const BAR_WIDTH: usize = 20;

/// Caller-supplied flags that change how the human view is rendered.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct RenderOptions<'a> {
    /// If true, append a per-wave breakdown under each plan.
    pub show_waves: bool,
    /// If `Some`, the human header notes that we are filtering by
    /// caller agent id (`--mine`). The actual filtering already
    /// happened in `status::run` before we got here.
    pub mine: Option<&'a str>,
}

/// Render the localized human view.
pub(crate) fn render_human(bundle: &Bundle, status: &StatusResponse, opts: RenderOptions<'_>) {
    println!("{}", bundle.t("status-header", &[]));
    if let Some(me) = opts.mine {
        println!("{}", bundle.t("status-mine-header", &[("agent", me)]));
    }

    if status.active_plans.is_empty() {
        println!("{}", bundle.t("status-active-empty", &[]));
    } else {
        println!("{}", bundle.t("status-active-header", &[]));
        for plan in &status.active_plans {
            print_plan(bundle, plan, opts);
        }
    }

    if status.recent_completed_plans.is_empty() {
        println!("{}", bundle.t("status-completed-empty", &[]));
    } else {
        println!("{}", bundle.t("status-completed-header", &[]));
        for plan in &status.recent_completed_plans {
            print_plan(bundle, plan, opts);
        }
    }

    if status.recent_completed_tasks.is_empty() {
        println!("{}", bundle.t("status-tasks-empty", &[]));
    } else {
        println!("{}", bundle.t("status-tasks-header", &[]));
        for task in &status.recent_completed_tasks {
            print_completed_task(bundle, task);
        }
    }
}

fn print_plan(bundle: &Bundle, plan: &PlanSummary, opts: RenderOptions<'_>) {
    let counts = &plan.tasks;
    println!(
        "{}",
        bundle.t(
            "status-plan-line",
            &[
                ("title", &plan.title),
                ("status", &plan.status),
                ("project", plan.project.as_deref().unwrap_or("-")),
                ("done", &counts.done.to_string()),
                ("total", &counts.total.to_string()),
            ],
        )
    );

    println!(
        "{}",
        bundle.t(
            "status-progress-line",
            &[
                ("bar", &progress_bar(counts.done, counts.total)),
                ("done", &counts.done.to_string()),
                ("total", &counts.total.to_string()),
            ],
        )
    );

    println!(
        "{}",
        bundle.t(
            "status-breakdown-line",
            &[
                ("done", &counts.done.to_string()),
                ("submitted", &counts.submitted.to_string()),
                ("in_progress", &counts.in_progress.to_string()),
                ("pending", &counts.pending.to_string()),
                ("failed", &counts.failed.to_string()),
                ("total", &counts.total.to_string()),
            ],
        )
    );

    let work = trim_description(plan.description.as_deref());
    println!("{}", bundle.t("status-work-line", &[("work", &work)]));

    let next = next_tasks_line(&plan.next_tasks);
    println!("{}", bundle.t("status-next-line", &[("tasks", &next)]));

    if opts.show_waves {
        print_wave_breakdown(bundle, &plan.next_tasks);
    }
}

fn print_completed_task(bundle: &Bundle, task: &CompletedTask) {
    println!(
        "{}",
        bundle.t(
            "status-task-line",
            &[
                ("title", &task.title),
                ("plan", &task.plan_title),
                ("project", task.project.as_deref().unwrap_or("-")),
            ],
        )
    );
}

fn print_wave_breakdown(bundle: &Bundle, tasks: &[TaskSummary]) {
    use std::collections::BTreeMap;
    let mut per_wave: BTreeMap<i64, TaskCounts> = BTreeMap::new();
    for task in tasks {
        let wave = task.wave.unwrap_or(0);
        let entry = per_wave.entry(wave).or_default();
        entry.total += 1;
        match task.status.as_deref() {
            Some("pending") => entry.pending += 1,
            Some("in_progress") => entry.in_progress += 1,
            Some("submitted") => entry.submitted += 1,
            Some("done") => entry.done += 1,
            Some("failed") => entry.failed += 1,
            _ => {}
        }
    }
    for (wave, c) in per_wave {
        println!(
            "{}",
            bundle.t(
                "status-wave-line",
                &[
                    ("wave", &wave.to_string()),
                    ("done", &c.done.to_string()),
                    ("submitted", &c.submitted.to_string()),
                    ("in_progress", &c.in_progress.to_string()),
                    ("pending", &c.pending.to_string()),
                    ("failed", &c.failed.to_string()),
                ],
            )
        );
    }
}

/// Trim a plan description to a fixed width, appending `…` when
/// the original was longer. Empty / missing descriptions render as
/// `-` so the line stays aligned in plain terminals.
pub(crate) fn trim_description(desc: Option<&str>) -> String {
    let raw = desc.map(str::trim).filter(|v| !v.is_empty()).unwrap_or("-");
    // Collapse newlines so the human view stays one line per plan.
    let one_line: String = raw
        .chars()
        .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
        .collect();
    if one_line.chars().count() <= DESCRIPTION_TRIM {
        return one_line;
    }
    let cut: String = one_line.chars().take(DESCRIPTION_TRIM).collect();
    format!("{cut}…")
}

/// Build the `[####....]` ASCII progress bar.
pub(crate) fn progress_bar(done: usize, total: usize) -> String {
    if total == 0 {
        return format!("[{}]", "·".repeat(BAR_WIDTH));
    }
    let filled = (done * BAR_WIDTH).saturating_div(total).min(BAR_WIDTH);
    let mut bar = String::with_capacity(BAR_WIDTH + 2);
    bar.push('[');
    for _ in 0..filled {
        bar.push('#');
    }
    for _ in filled..BAR_WIDTH {
        bar.push('.');
    }
    bar.push(']');
    bar
}

fn next_tasks_line(tasks: &[TaskSummary]) -> String {
    let titles: Vec<&str> = tasks.iter().take(3).map(|t| t.title.as_str()).collect();
    if titles.is_empty() {
        "-".to_string()
    } else {
        titles.join(" · ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_keeps_short_descriptions_unchanged() {
        assert_eq!(trim_description(Some("hi")), "hi");
        assert_eq!(trim_description(None), "-");
        assert_eq!(trim_description(Some("   ")), "-");
    }

    #[test]
    fn trim_truncates_with_ellipsis() {
        let long = "a".repeat(200);
        let trimmed = trim_description(Some(&long));
        assert!(trimmed.ends_with('…'));
        assert_eq!(trimmed.chars().count(), DESCRIPTION_TRIM + 1);
    }

    #[test]
    fn trim_collapses_embedded_newlines() {
        let s = "line one\nline two";
        assert_eq!(trim_description(Some(s)), "line one line two");
    }

    #[test]
    fn progress_bar_zero_total_renders_empty_track() {
        let bar = progress_bar(0, 0);
        assert!(bar.starts_with('['));
        assert!(bar.ends_with(']'));
        assert_eq!(bar.chars().filter(|c| *c == '·').count(), BAR_WIDTH);
    }

    #[test]
    fn progress_bar_full_when_done_equals_total() {
        let bar = progress_bar(50, 50);
        assert_eq!(bar.matches('#').count(), BAR_WIDTH);
    }

    #[test]
    fn progress_bar_half() {
        let bar = progress_bar(1, 2);
        assert_eq!(bar.matches('#').count(), BAR_WIDTH / 2);
        assert_eq!(bar.matches('.').count(), BAR_WIDTH / 2);
    }

    #[test]
    fn next_tasks_line_caps_at_three() {
        let tasks: Vec<TaskSummary> = (0..5)
            .map(|i| TaskSummary {
                title: format!("t{i}"),
                status: None,
                agent_id: None,
                wave: None,
                sequence: None,
            })
            .collect();
        let line = next_tasks_line(&tasks);
        assert_eq!(line, "t0 · t1 · t2");
    }

    #[test]
    fn next_tasks_line_dash_when_empty() {
        assert_eq!(next_tasks_line(&[]), "-");
    }
}
