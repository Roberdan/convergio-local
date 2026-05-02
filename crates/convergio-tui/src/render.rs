//! Top-level layout and footer.
//!
//! Splits the frame into header (title bar), body (4-pane grid), and
//! footer (status line). Each pane delegates to its own renderer in
//! [`crate::panes`].

use crate::header_banner;
use crate::panes;
use crate::state::{version_drift, AppState, Connection, Pane, BINARY_VERSION};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Draw one frame.
pub fn root(f: &mut Frame, state: &AppState) {
    let area = f.area();
    let header_h = header_banner::header_height(area.width);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_h), // header
            Constraint::Min(8),           // body
            Constraint::Length(1),        // footer
        ])
        .split(area);
    draw_header(f, chunks[0], state);
    draw_body(f, chunks[1], state);
    draw_footer(f, chunks[2], state);
}

fn draw_header(f: &mut Frame, area: Rect, state: &AppState) {
    header_banner::render(f, area, &header_subtitle(state));
}

fn header_subtitle(state: &AppState) -> String {
    let plans = state.plans.len();
    let tasks = state.tasks.len();
    let agents = state.agents.len();
    let prs = state.prs.len();
    let version_part = match version_drift(state.daemon_version.as_deref()) {
        Some(daemon) => format!(" ⚠ binary v{BINARY_VERSION} ≠ daemon v{daemon} (run cvg update)"),
        None => match state.daemon_version.as_deref() {
            Some(d) => format!(" v{d}"),
            None => format!(" v{BINARY_VERSION}"),
        },
    };
    format!("plans:{plans} tasks:{tasks} agents:{agents} prs:{prs}{version_part}")
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
    let pane_name = format!("pane: {}", state.focus.label());
    let line = Line::from(vec![
        conn,
        Span::raw(" · "),
        audit,
        Span::raw(" · "),
        Span::raw(last),
        Span::raw(" · "),
        Span::styled(
            pane_name,
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" · "),
        Span::styled(
            "q quit  r refresh  Tab pane  j/k row",
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

/// Common helper: build a bordered block with a title that
/// highlights when its pane is focused.
///
/// Focus is signalled three independent ways so it stays visible on
/// small / low-contrast / no-truecolour terminals:
/// 1. A `▶` glyph prefix in the title (works without colour).
/// 2. Reverse-video on the title (background swap).
/// 3. Cyan bold border, dim border otherwise.
pub fn pane_block(title: &str, focused: bool) -> Block<'static> {
    let prefix = if focused { "▶ " } else { "  " };
    let owned_title = format!("{prefix}{title}");
    let title_style = if focused {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let border_style = if focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(owned_title, title_style))
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
        let dump = buf.content().iter().map(|c| c.symbol()).collect::<String>();
        assert!(
            dump.contains("█"),
            "banner block glyphs should be rendered: {dump:?}"
        );
        assert!(
            dump.contains(BINARY_VERSION),
            "subtitle should mention the binary version: {dump:?}"
        );
    }
}
