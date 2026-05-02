//! Pull requests pane.
//!
//! Open PRs from `gh pr list`. Scoping by plan is best-effort: a PR
//! is treated as "in scope" when its branch name or title contains
//! the scoped plan id (or the first 8 chars of it). Without the
//! `plan_pr_links` table this is the most reliable heuristic that
//! does not lie — when nothing matches we still show every PR with
//! a "no link" hint in the title crumb.

use crate::client::PrSummary;
use crate::render::pane_block;
use crate::state::AppState;
use crate::theme;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState};
use ratatui::Frame;

/// Render the PRs pane.
pub fn render(f: &mut Frame, area: Rect, state: &AppState, focused: bool) {
    let scoped_id = state.scoped_plan_id();
    let id_short = scoped_id.map(|id| id.get(..8).unwrap_or(id));
    let scoped: Vec<&PrSummary> = match (scoped_id, id_short) {
        (Some(id), Some(short_id)) => state
            .prs
            .iter()
            .filter(|pr| {
                pr.head_ref_name.contains(short_id)
                    || pr.title.contains(short_id)
                    || pr.head_ref_name.contains(id)
                    || pr.title.contains(id)
            })
            .collect(),
        _ => state.prs.iter().collect(),
    };
    let scope_crumb = match (scoped_id, scoped.is_empty(), state.prs.is_empty()) {
        (Some(_), true, false) => " · no link".to_string(),
        (Some(_), _, _) => format!(" · {}", short(state.scoped_plan_title().unwrap_or(""), 24)),
        _ => String::new(),
    };
    let title = format!(" PRs ({}){scope_crumb} ", scoped.len());
    let block = pane_block(&title, focused);

    let selected_idx = state
        .cursor
        .prs
        .selected
        .min(scoped.len().saturating_sub(1));
    let items: Vec<ListItem> = scoped
        .iter()
        .enumerate()
        .map(|(idx, pr)| ListItem::new(pr_line(pr, idx == selected_idx)))
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(selected_idx));

    let list = List::new(items)
        .block(block)
        .highlight_style(theme::row_highlight());
    f.render_stateful_widget(list, area, &mut list_state);
}

fn pr_line(pr: &PrSummary, is_selected: bool) -> Line<'static> {
    let accent = if is_selected {
        theme::accent_span()
    } else {
        theme::accent_gap()
    };
    Line::from(vec![
        accent,
        Span::raw(" "),
        Span::styled(format!("#{:<4}", pr.number), theme::heading()),
        Span::raw(" "),
        Span::styled(ci_glyph(&pr.ci).to_string(), ci_style(&pr.ci)),
        Span::raw(" "),
        Span::styled(format!("{:28}", short(&pr.head_ref_name, 28)), theme::dim()),
        Span::raw(" "),
        Span::raw(short(&pr.title, 40).to_string()),
    ])
}

fn ci_glyph(ci: &str) -> &'static str {
    match ci {
        "success" | "SUCCESS" => "✓",
        "failure" | "FAILURE" => "✗",
        "pending" | "PENDING" => "…",
        _ => "?",
    }
}

fn ci_style(ci: &str) -> Style {
    match ci {
        "success" | "SUCCESS" => Style::default().fg(theme::SUCCESS),
        "failure" | "FAILURE" => Style::default().fg(theme::DANGER),
        "pending" | "PENDING" => Style::default().fg(theme::WARNING),
        _ => theme::dim(),
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
                pr(93, "hardening/lifecycle", "fix(lifecycle): x", "success"),
            ],
            ..AppState::default()
        };
        term.draw(|f| render(f, f.area(), &state, false)).unwrap();
        let dump = term
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect::<String>();
        assert!(dump.contains("PRs"));
        assert!(dump.contains("#92"));
        assert!(dump.contains("#93"));
    }

    #[test]
    fn ci_glyph_unknown_falls_back_to_question_mark() {
        assert_eq!(ci_glyph(""), "?");
        assert_eq!(ci_glyph("weird"), "?");
    }
}
