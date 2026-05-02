//! Pull requests pane.
//!
//! One row per open PR returned by `gh pr list`. CI conclusion is
//! summarised across the rollup checks (failure → ✗, pending → …,
//! success → ✓).

use crate::client::PrSummary;
use crate::render::pane_block;
use crate::state::AppState;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState};
use ratatui::Frame;

/// Render the PRs pane.
pub fn render(f: &mut Frame, area: Rect, state: &AppState, focused: bool) {
    let title = format!(" PRs ({}) ", state.prs.len());
    let block = pane_block(&title, focused);

    let items: Vec<ListItem> = state
        .prs
        .iter()
        .map(|pr| ListItem::new(pr_line(pr)))
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(
        state
            .cursor
            .prs
            .selected
            .min(state.prs.len().saturating_sub(1)),
    ));

    let list = List::new(items).block(block).highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );
    f.render_stateful_widget(list, area, &mut list_state);
}

fn pr_line(pr: &PrSummary) -> Line<'_> {
    Line::from(vec![
        Span::styled(
            format!("#{:<4}", pr.number),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(ci_glyph(&pr.ci).to_string(), ci_style(&pr.ci)),
        Span::raw(" "),
        Span::styled(
            format!("{:32}", truncate(&pr.head_ref_name, 32)),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw(" "),
        Span::raw(truncate(&pr.title, 40).to_string()),
    ])
}

fn ci_glyph(ci: &str) -> &'static str {
    match ci {
        "success" => "✓",
        "failure" => "✗",
        "pending" => "…",
        _ => "?",
    }
}

fn ci_style(ci: &str) -> Style {
    match ci {
        "success" => Style::default().fg(Color::Green),
        "failure" => Style::default().fg(Color::Red),
        "pending" => Style::default().fg(Color::Yellow),
        _ => Style::default().fg(Color::DarkGray),
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

    fn pr(n: i64, branch: &str, title: &str, ci: &str) -> PrSummary {
        PrSummary {
            number: n,
            title: title.into(),
            head_ref_name: branch.into(),
            ci: ci.into(),
        }
    }

    #[test]
    fn render_prs_includes_number_and_ci_glyph() {
        let backend = TestBackend::new(120, 6);
        let mut term = Terminal::new(backend).unwrap();
        let state = AppState {
            prs: vec![
                pr(92, "hardening/mcp-e2e", "test(mcp): coverage", "failure"),
                pr(93, "hardening/lifecycle", "fix(lifecycle): ...", "success"),
            ],
            ..AppState::default()
        };
        term.draw(|f| render(f, f.area(), &state, false)).unwrap();
        let buf = term.backend().buffer();
        let dump = buf.content().iter().map(|c| c.symbol()).collect::<String>();
        assert!(dump.contains("PRs"));
        assert!(dump.contains("#92"));
        assert!(dump.contains("#93"));
        assert!(
            dump.contains("✗") || dump.contains("X"),
            "failure glyph missing: {dump:?}"
        );
        assert!(
            dump.contains("✓") || dump.contains("v"),
            "success glyph missing"
        );
    }

    #[test]
    fn ci_glyph_unknown_falls_back_to_question_mark() {
        assert_eq!(ci_glyph(""), "?");
        assert_eq!(ci_glyph("weird"), "?");
    }
}
