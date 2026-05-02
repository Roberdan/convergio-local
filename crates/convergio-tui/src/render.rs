//! Top-level layout and footer.
//!
//! Splits the frame into header (title bar), body (4-pane grid), and
//! footer (status line). Each pane delegates to its own renderer in
//! [`crate::panes`].

use crate::panes;
use crate::state::{AppState, Connection, Pane};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Draw one frame.
pub fn root(f: &mut Frame, state: &AppState) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Min(8),    // body
            Constraint::Length(1), // footer
        ])
        .split(area);
    draw_header(f, chunks[0], state);
    draw_body(f, chunks[1], state);
    draw_footer(f, chunks[2], state);
}

fn draw_header(f: &mut Frame, area: Rect, state: &AppState) {
    let agents = state.agents.len();
    let plans = state.plans.len();
    let title = format!(
        "convergio · plans:{plans} agents:{agents} prs:{prs} tasks:{tasks}",
        prs = state.prs.len(),
        tasks = state.tasks.len(),
    );
    let p = Paragraph::new(Span::styled(
        title,
        Style::default().add_modifier(Modifier::BOLD),
    ));
    f.render_widget(p, area);
}

fn draw_body(f: &mut Frame, area: Rect, state: &AppState) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[0]);
    let bot = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    panes::plans::render(f, top[0], state, focused(state, Pane::Plans));
    panes::tasks::render(f, top[1], state, focused(state, Pane::Tasks));
    panes::agents::render(f, bot[0], state, focused(state, Pane::Agents));
    panes::prs::render(f, bot[1], state, focused(state, Pane::Prs));
}

fn focused(state: &AppState, pane: Pane) -> bool {
    state.focus == pane
}

fn draw_footer(f: &mut Frame, area: Rect, state: &AppState) {
    let conn = match state.connection {
        Connection::Initial => Span::styled("connecting", Style::default().fg(Color::Yellow)),
        Connection::Connected => Span::styled("connected", Style::default().fg(Color::Green)),
        Connection::Disconnected => Span::styled("disconnected", Style::default().fg(Color::Red)),
    };
    let audit = match state.audit_ok {
        Some(true) => Span::styled("audit ✓", Style::default().fg(Color::Green)),
        Some(false) => Span::styled("audit ✗", Style::default().fg(Color::Red)),
        None => Span::raw("audit ?"),
    };
    let last = match state.last_refresh {
        Some(t) => format!("last {}", t.format("%H:%M:%S")),
        None => "last –".into(),
    };
    let line = Line::from(vec![
        conn,
        Span::raw(" · "),
        audit,
        Span::raw(" · "),
        Span::raw(last),
        Span::raw(" · "),
        Span::styled(
            "q quit  r refresh  Tab pane  j/k row",
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

/// Common helper: build a bordered block with a title that highlights
/// when its pane is focused.
pub fn pane_block<'a>(title: &'a str, focused: bool) -> Block<'a> {
    let style = if focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    Block::default()
        .borders(Borders::ALL)
        .border_style(if focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        })
        .title(Span::styled(title, style))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn root_renders_without_panic_on_empty_state() {
        let backend = TestBackend::new(120, 40);
        let mut term = Terminal::new(backend).unwrap();
        let state = AppState::default();
        term.draw(|f| root(f, &state)).unwrap();
        let buf = term.backend().buffer();
        let header = buf
            .content()
            .iter()
            .take(120)
            .map(|c| c.symbol())
            .collect::<String>();
        assert!(
            header.contains("convergio"),
            "header missing convergio brand: {header:?}"
        );
    }
}
