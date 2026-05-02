//! Active tasks pane.
//!
//! One row per active task across every plan. Active means
//! `in_progress` or `submitted` — tasks the system is currently
//! waiting on. Sorted by status (in_progress first), then plan id.

use crate::client::TaskSummary;
use crate::render::pane_block;
use crate::state::AppState;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState};
use ratatui::Frame;

/// Render the Active Tasks pane.
pub fn render(f: &mut Frame, area: Rect, state: &AppState, focused: bool) {
    let title = format!(" Active tasks ({}) ", state.tasks.len());
    let block = pane_block(&title, focused);

    let mut sorted = state.tasks.clone();
    sorted.sort_by_key(|t| status_priority(&t.status));

    let items: Vec<ListItem> = sorted.iter().map(|t| ListItem::new(task_line(t))).collect();

    let mut list_state = ListState::default();
    list_state.select(Some(
        state
            .cursor
            .tasks
            .selected
            .min(state.tasks.len().saturating_sub(1)),
    ));

    let list = List::new(items).block(block).highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );
    f.render_stateful_widget(list, area, &mut list_state);
}

fn task_line(t: &TaskSummary) -> Line<'_> {
    let owner = t.agent_id.as_deref().unwrap_or("-");
    Line::from(vec![
        Span::styled(
            short_id(&t.id).to_string(),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw(" "),
        Span::styled(format!("{:12}", &t.status), status_style(&t.status)),
        Span::raw(" "),
        Span::styled(
            format!("{:18}", short_id(owner)),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw(" "),
        Span::raw(truncate(&t.title, 60).to_string()),
    ])
}

fn status_priority(status: &str) -> u8 {
    match status {
        "in_progress" => 0,
        "submitted" => 1,
        "pending" => 2,
        "failed" => 3,
        "done" => 4,
        _ => 5,
    }
}

fn status_style(status: &str) -> Style {
    match status {
        "in_progress" => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        "submitted" => Style::default().fg(Color::Cyan),
        "pending" => Style::default().fg(Color::DarkGray),
        "done" => Style::default().fg(Color::Green),
        "failed" => Style::default().fg(Color::Red),
        _ => Style::default(),
    }
}

fn short_id(id: &str) -> &str {
    id.get(..8).unwrap_or(id)
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
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn task(id: &str, status: &str) -> TaskSummary {
        TaskSummary {
            id: id.into(),
            plan_id: "p".into(),
            title: format!("title-{id}"),
            status: status.into(),
            agent_id: Some("claude-code-roberdan".into()),
        }
    }

    #[test]
    fn status_priority_orders_in_progress_first() {
        let mut v = vec!["done", "in_progress", "submitted", "pending"];
        v.sort_by_key(|s| status_priority(s));
        assert_eq!(v, vec!["in_progress", "submitted", "pending", "done"]);
    }

    #[test]
    fn render_tasks_includes_status_and_owner() {
        let backend = TestBackend::new(120, 8);
        let mut term = Terminal::new(backend).unwrap();
        let state = AppState {
            tasks: vec![
                task("aaaaaaaa11", "in_progress"),
                task("bbbbbbbb22", "submitted"),
            ],
            ..AppState::default()
        };
        term.draw(|f| render(f, f.area(), &state, true)).unwrap();
        let buf = term.backend().buffer();
        let dump = buf.content().iter().map(|c| c.symbol()).collect::<String>();
        assert!(dump.contains("Active tasks"));
        assert!(dump.contains("in_progress"));
        assert!(dump.contains("submitted"));
        assert!(dump.contains("claude-c"), "agent id prefix missing");
    }

    #[test]
    fn short_id_safe_on_short_strings() {
        assert_eq!(short_id("abc"), "abc");
        assert_eq!(short_id("abcdefghij"), "abcdefgh");
    }
}
