//! Plans pane.
//!
//! One row per plan, with a status badge and the count of currently
//! active tasks (in_progress + submitted) for that plan. No
//! client-side stats invention — every number rendered is recomputed
//! from the daemon-provided dataset on the same refresh tick.

use crate::client::Plan;
use crate::render::pane_block;
use crate::state::AppState;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState};
use ratatui::Frame;

/// Render the Plans pane into `area`.
pub fn render(f: &mut Frame, area: Rect, state: &AppState, focused: bool) {
    let title = format!(" Plans ({}) ", state.plans.len());
    let block = pane_block(&title, focused);

    let items: Vec<ListItem> = state
        .plans
        .iter()
        .map(|p| ListItem::new(plan_lines(p, state)))
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(
        state
            .cursor
            .plans
            .selected
            .min(state.plans.len().saturating_sub(1)),
    ));

    let list = List::new(items).block(block).highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );
    f.render_stateful_widget(list, area, &mut list_state);
}

fn plan_lines(p: &Plan, state: &AppState) -> Vec<Line<'static>> {
    // Active task count for this plan (in_progress + submitted).
    let active = state.tasks.iter().filter(|t| t.plan_id == p.id).count();

    let title_line = Line::from(vec![
        Span::styled(
            truncate(&p.title, 50).to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(format!("[{}]", p.status), status_style(&p.status)),
        Span::raw("  "),
        Span::styled(
            format!("active:{active}"),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let project = p.project.as_deref().unwrap_or("-").to_string();
    let updated = p.updated_at.get(..16).unwrap_or(&p.updated_at).to_string();
    let meta_line = Line::from(vec![Span::styled(
        format!("  project: {project}  updated: {updated}"),
        Style::default().fg(Color::DarkGray),
    )]);

    vec![title_line, meta_line]
}

fn status_style(status: &str) -> Style {
    match status {
        "active" => Style::default().fg(Color::Green),
        "draft" => Style::default().fg(Color::Yellow),
        "completed" => Style::default().fg(Color::DarkGray),
        "cancelled" | "failed" => Style::default().fg(Color::Red),
        _ => Style::default(),
    }
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        // truncate at byte boundary safely
        let mut end = max;
        while !s.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        &s[..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{Plan, TaskSummary};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn state_with(plans: Vec<Plan>, tasks: Vec<TaskSummary>) -> AppState {
        AppState {
            plans,
            tasks,
            ..AppState::default()
        }
    }

    #[test]
    fn truncate_handles_unicode_safely() {
        let s = "abcdèfgh";
        let t = truncate(s, 4);
        // Should not panic even when max lands on a multi-byte boundary.
        assert!(s.starts_with(t));
    }

    #[test]
    fn render_plans_lists_titles() {
        let backend = TestBackend::new(80, 12);
        let mut term = Terminal::new(backend).unwrap();
        let state = state_with(
            vec![Plan {
                id: "p1".into(),
                title: "v0.4 — Distribution".into(),
                project: Some("convergio".into()),
                status: "active".into(),
                updated_at: "2026-05-02T12:00:00Z".into(),
            }],
            vec![],
        );
        term.draw(|f| render(f, f.area(), &state, true)).unwrap();
        let buf = term.backend().buffer();
        let dump = buf.content().iter().map(|c| c.symbol()).collect::<String>();
        assert!(dump.contains("Plans"), "title missing: {dump:?}");
        assert!(dump.contains("v0.4 — Distribution"), "plan title missing");
    }

    #[test]
    fn render_plans_marks_active_count_per_plan() {
        let backend = TestBackend::new(100, 6);
        let mut term = Terminal::new(backend).unwrap();
        let state = state_with(
            vec![Plan {
                id: "p1".into(),
                title: "p1".into(),
                project: None,
                status: "draft".into(),
                updated_at: "2026-05-02".into(),
            }],
            vec![TaskSummary {
                id: "t1".into(),
                plan_id: "p1".into(),
                title: "do".into(),
                status: "in_progress".into(),
                agent_id: None,
            }],
        );
        term.draw(|f| render(f, f.area(), &state, false)).unwrap();
        let buf = term.backend().buffer();
        let dump = buf.content().iter().map(|c| c.symbol()).collect::<String>();
        assert!(dump.contains("active:1"), "active count missing");
    }
}
