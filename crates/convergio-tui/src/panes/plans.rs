//! Plans pane.
//!
//! Master pane in the lazygit-style scoped master/detail layout
//! (ADR-0029). Whichever plan the cursor sits on is the *scope* the
//! Tasks / Agents / PRs panes filter against, so as the user moves
//! up/down here the rest of the dashboard re-renders.

use crate::client::Plan;
use crate::render::pane_block;
use crate::state::AppState;
use crate::theme;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState};
use ratatui::Frame;

/// Render the Plans pane into `area`.
pub fn render(f: &mut Frame, area: Rect, state: &AppState, focused: bool) {
    let title = format!(" Plans ({}) ", state.plans.len());
    let block = pane_block(&title, focused);

    let selected_idx = state
        .cursor
        .plans
        .selected
        .min(state.plans.len().saturating_sub(1));
    let items: Vec<ListItem> = state
        .plans
        .iter()
        .enumerate()
        .map(|(idx, p)| ListItem::new(plan_lines(p, state, idx == selected_idx)))
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(selected_idx));

    let list = List::new(items)
        .block(block)
        .highlight_style(theme::row_highlight());
    f.render_stateful_widget(list, area, &mut list_state);
}

fn plan_lines(p: &Plan, state: &AppState, is_selected: bool) -> Vec<Line<'static>> {
    let active = state.tasks.iter().filter(|t| t.plan_id == p.id).count();
    let (status_glyph, status_style) = theme::status_pill(&p.status);
    let accent = if is_selected {
        theme::accent_span()
    } else {
        theme::accent_gap()
    };

    let title_line = Line::from(vec![
        accent.clone(),
        Span::raw(" "),
        status_glyph,
        Span::raw(" "),
        Span::styled(truncate(&p.title, 48).to_string(), theme::heading()),
        Span::raw("  "),
        Span::styled(format!("[{}]", p.status), status_style),
        Span::raw("  "),
        Span::styled(format!("active:{active}"), theme::dim()),
    ]);

    let project = p.project.as_deref().unwrap_or("-").to_string();
    let updated = p.updated_at.get(..16).unwrap_or(&p.updated_at).to_string();
    let meta_line = Line::from(vec![
        accent,
        Span::raw("   "),
        Span::styled(
            format!("project: {project}  updated: {updated}"),
            theme::dim(),
        ),
    ]);

    vec![title_line, meta_line]
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
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
        assert!(s.starts_with(t));
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
        let dump = term
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect::<String>();
        assert!(dump.contains("active:1"));
        assert!(dump.contains("Plans"));
    }
}
