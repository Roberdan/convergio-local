//! Agents pane.
//!
//! One row per registered agent: id, kind (shell/claude/copilot),
//! status (idle/working/terminated/...), and the time since last
//! heartbeat.

use crate::client::RegistryAgent;
use crate::render::pane_block;
use crate::state::AppState;
use chrono::Utc;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState};
use ratatui::Frame;

/// Render the Agents pane.
pub fn render(f: &mut Frame, area: Rect, state: &AppState, focused: bool) {
    let title = format!(" Agents ({}) ", state.agents.len());
    let block = pane_block(&title, focused);

    let items: Vec<ListItem> = state
        .agents
        .iter()
        .map(|a| ListItem::new(agent_line(a)))
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(
        state
            .cursor
            .agents
            .selected
            .min(state.agents.len().saturating_sub(1)),
    ));

    let list = List::new(items).block(block).highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );
    f.render_stateful_widget(list, area, &mut list_state);
}

fn agent_line(a: &RegistryAgent) -> Line<'_> {
    let status = a.status.as_deref().unwrap_or("?");
    let dot = match status {
        "working" | "active" => Span::styled("◉", Style::default().fg(Color::Green)),
        "idle" => Span::styled("◉", Style::default().fg(Color::DarkGray)),
        "terminated" | "retired" => Span::styled("◌", Style::default().fg(Color::Red)),
        _ => Span::raw("·"),
    };
    Line::from(vec![
        dot,
        Span::raw(" "),
        Span::styled(
            format!("{:32}", truncate(&a.id, 32)),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(format!("{:10}", &a.kind), Style::default().fg(Color::Cyan)),
        Span::raw(" "),
        Span::styled(format!("{:12}", status), status_style(status)),
        Span::raw(" "),
        Span::styled(
            heartbeat_age(a.last_heartbeat_at.as_deref()),
            Style::default().fg(Color::DarkGray),
        ),
    ])
}

fn status_style(status: &str) -> Style {
    match status {
        "idle" => Style::default().fg(Color::DarkGray),
        "working" | "active" => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        "terminated" | "retired" => Style::default().fg(Color::Red),
        _ => Style::default(),
    }
}

fn heartbeat_age(last: Option<&str>) -> String {
    let raw = match last {
        Some(s) if !s.is_empty() => s,
        _ => return "no heartbeat".into(),
    };
    let parsed = chrono::DateTime::parse_from_rfc3339(raw);
    match parsed {
        Ok(t) => {
            let secs = (Utc::now() - t.with_timezone(&Utc)).num_seconds();
            if secs < 0 {
                return "future".into();
            }
            if secs < 60 {
                return format!("{secs}s ago");
            }
            if secs < 3600 {
                return format!("{}m ago", secs / 60);
            }
            format!("{}h ago", secs / 3600)
        }
        Err(_) => "?".into(),
    }
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

    fn agent(id: &str, kind: &str, status: &str) -> RegistryAgent {
        RegistryAgent {
            id: id.into(),
            kind: kind.into(),
            status: Some(status.into()),
            last_heartbeat_at: None,
        }
    }

    #[test]
    fn render_agents_lists_id_kind_status() {
        let backend = TestBackend::new(100, 6);
        let mut term = Terminal::new(backend).unwrap();
        let state = AppState {
            agents: vec![
                agent("claude-code-roberdan", "claude", "idle"),
                agent("copilot-overnight", "copilot", "terminated"),
            ],
            ..AppState::default()
        };
        term.draw(|f| render(f, f.area(), &state, true)).unwrap();
        let buf = term.backend().buffer();
        let dump = buf.content().iter().map(|c| c.symbol()).collect::<String>();
        assert!(dump.contains("Agents"));
        assert!(dump.contains("claude-code-roberdan"));
        assert!(dump.contains("idle"));
        assert!(dump.contains("terminated"));
    }

    #[test]
    fn heartbeat_age_falls_back_when_missing() {
        assert_eq!(heartbeat_age(None), "no heartbeat");
        assert_eq!(heartbeat_age(Some("")), "no heartbeat");
        assert_eq!(heartbeat_age(Some("garbage")), "?");
    }
}
