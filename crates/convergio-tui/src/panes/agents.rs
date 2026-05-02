//! Agents pane.
//!
//! Filtered against [`AppState::scoped_agents`] — when a plan is the
//! current scope, only agents that own at least one task in that
//! plan are listed. With no scope, every registered agent is shown.

use crate::client::RegistryAgent;
use crate::render::pane_block;
use crate::state::AppState;
use crate::theme;
use chrono::Utc;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState};
use ratatui::Frame;

/// Render the Agents pane.
pub fn render(f: &mut Frame, area: Rect, state: &AppState, focused: bool) {
    let scoped: Vec<RegistryAgent> = state.scoped_agents().into_iter().cloned().collect();
    let scope_crumb = state
        .scoped_plan_title()
        .map(|t| format!(" · {}", short(t, 24)))
        .unwrap_or_default();
    let title = format!(" Agents ({}){scope_crumb} ", scoped.len());
    let block = pane_block(&title, focused);

    let selected_idx = state
        .cursor
        .agents
        .selected
        .min(scoped.len().saturating_sub(1));
    let items: Vec<ListItem> = scoped
        .iter()
        .enumerate()
        .map(|(idx, a)| ListItem::new(agent_line(a, idx == selected_idx)))
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(selected_idx));

    let list = List::new(items)
        .block(block)
        .highlight_style(theme::row_highlight());
    f.render_stateful_widget(list, area, &mut list_state);
}

fn agent_line(a: &RegistryAgent, is_selected: bool) -> Line<'static> {
    let status = a.status.as_deref().unwrap_or("?");
    let (status_glyph, status_style) = theme::status_pill(status);
    let accent = if is_selected {
        theme::accent_span()
    } else {
        theme::accent_gap()
    };
    Line::from(vec![
        accent,
        Span::raw(" "),
        status_glyph,
        Span::raw(" "),
        Span::styled(format!("{:32}", short(&a.id, 32)), theme::heading()),
        Span::raw(" "),
        Span::styled(format!("{:10}", &a.kind), theme::dim()),
        Span::raw(" "),
        Span::styled(format!("{:12}", status), status_style),
        Span::raw(" "),
        Span::styled(heartbeat_age(a.last_heartbeat_at.as_deref()), theme::dim()),
    ])
}

fn heartbeat_age(last: Option<&str>) -> String {
    let raw = match last {
        Some(s) if !s.is_empty() => s,
        _ => return "no heartbeat".into(),
    };
    match chrono::DateTime::parse_from_rfc3339(raw) {
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

fn short(s: &str, max: usize) -> &str {
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
        let backend = TestBackend::new(110, 6);
        let mut term = Terminal::new(backend).unwrap();
        let state = AppState {
            agents: vec![
                agent("claude-code-roberdan", "claude", "idle"),
                agent("copilot-overnight", "copilot", "terminated"),
            ],
            ..AppState::default()
        };
        term.draw(|f| render(f, f.area(), &state, true)).unwrap();
        let dump = term
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect::<String>();
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
