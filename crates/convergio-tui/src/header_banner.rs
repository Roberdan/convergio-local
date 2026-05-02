//! Adaptive gradient header for the dashboard.
//!
//! Three layout tiers, picked by available width:
//!
//! 1. **Side-by-side** (`width >= 100`): the ANSI-shadow CONVERGIO
//!    wordmark on the left, the stats column right-aligned to the
//!    far edge — one row per stat.
//! 2. **Stacked** (`width >= 75`): the wordmark on top, a single
//!    stats line under it.
//! 3. **Compact** (`width < 75`): one line with a styled wordmark
//!    plus the stats — keeps `cvg dash` usable on narrow shells.
//!
//! The wordmark uses cyan→magenta `Color::Rgb` gradient. Terminals
//! without true-colour fall back to ratatui's nearest 256-colour
//! mapping (CONSTITUTION P3: information conveyed by colour is also
//! conveyed by glyph/label).

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

/// 6-row ANSI Shadow wordmark. ~73 cols wide.
const WORDMARK: &[&str] = &[
    " ██████╗ ██████╗ ███╗   ██╗██╗   ██╗███████╗██████╗  ██████╗ ██╗ ██████╗ ",
    "██╔════╝██╔═══██╗████╗  ██║██║   ██║██╔════╝██╔══██╗██╔════╝ ██║██╔═══██╗",
    "██║     ██║   ██║██╔██╗ ██║██║   ██║█████╗  ██████╔╝██║  ███╗██║██║   ██║",
    "██║     ██║   ██║██║╚██╗██║╚██╗ ██╔╝██╔══╝  ██╔══██╗██║   ██║██║██║   ██║",
    "╚██████╗╚██████╔╝██║ ╚████║ ╚████╔╝ ███████╗██║  ██║╚██████╔╝██║╚██████╔╝",
    " ╚═════╝ ╚═════╝ ╚═╝  ╚═══╝  ╚═══╝  ╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚═╝ ╚═════╝ ",
];

const WORDMARK_WIDTH: u16 = 73;
const STATS_COLUMN_WIDTH: u16 = 24;
const SIDE_BY_SIDE_MIN_WIDTH: u16 = WORDMARK_WIDTH + 2 + STATS_COLUMN_WIDTH;
const STACKED_MIN_WIDTH: u16 = WORDMARK_WIDTH + 2;

/// Height of the side-by-side / stacked banner (6 rows for the
/// wordmark + 1 row for the stats line in stacked mode).
pub const BANNER_HEIGHT: u16 = 7;

/// Height of the compact (single-line) header.
pub const COMPACT_HEIGHT: u16 = 1;

/// Returns the height the header should reserve for the given
/// terminal `width`.
pub fn header_height(width: u16) -> u16 {
    if width >= STACKED_MIN_WIDTH {
        BANNER_HEIGHT
    } else {
        COMPACT_HEIGHT
    }
}

/// Render the header into `area`. `stats` is the per-line stats
/// column (e.g. `["plans:32", "tasks:99", ...]`); in compact and
/// stacked modes the lines are joined with spacers.
pub fn render(f: &mut Frame, area: Rect, stats: &[String]) {
    if area.width >= SIDE_BY_SIDE_MIN_WIDTH && area.height >= BANNER_HEIGHT {
        render_side_by_side(f, area, stats);
    } else if area.width >= STACKED_MIN_WIDTH && area.height >= BANNER_HEIGHT {
        render_stacked(f, area, stats);
    } else {
        render_compact(f, area, stats);
    }
}

fn render_side_by_side(f: &mut Frame, area: Rect, stats: &[String]) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(WORDMARK_WIDTH),
            Constraint::Min(STATS_COLUMN_WIDTH),
        ])
        .split(area);
    f.render_widget(Paragraph::new(banner_lines()), chunks[0]);
    f.render_widget(stats_column(stats, chunks[1].height as usize), chunks[1]);
}

fn render_stacked(f: &mut Frame, area: Rect, stats: &[String]) {
    let mut lines = banner_lines();
    lines.push(Line::from(Span::styled(
        stats.join("  ·  "),
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )));
    f.render_widget(Paragraph::new(lines), area);
}

fn render_compact(f: &mut Frame, area: Rect, stats: &[String]) {
    let line = Line::from(vec![
        Span::styled(
            "▌ CONVERGIO ▐",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            stats.join("  "),
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

fn banner_lines() -> Vec<Line<'static>> {
    let total = WORDMARK
        .iter()
        .map(|r| r.chars().count())
        .max()
        .unwrap_or(1);
    WORDMARK
        .iter()
        .map(|row| line_with_gradient(row, total))
        .collect()
}

/// Right-aligned stats column. Each line is right-aligned so the
/// visual right edge of the column lines up with the right edge of
/// the screen — the way htop / k9s do it.
fn stats_column(stats: &[String], height: usize) -> Paragraph<'static> {
    let mut lines: Vec<Line<'static>> = stats
        .iter()
        .map(|s| {
            Line::from(Span::styled(
                s.clone(),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ))
            .right_aligned()
        })
        .collect();
    while lines.len() < height {
        lines.push(Line::raw(""));
    }
    Paragraph::new(lines)
}

fn line_with_gradient(row: &str, total_cols: usize) -> Line<'static> {
    let chars: Vec<char> = row.chars().collect();
    let mut spans = Vec::with_capacity(chars.len());
    for (idx, ch) in chars.iter().enumerate() {
        if *ch == ' ' {
            spans.push(Span::raw(" "));
            continue;
        }
        let (r, g, b) = gradient_at(idx, total_cols.max(1));
        spans.push(Span::styled(
            ch.to_string(),
            Style::default()
                .fg(Color::Rgb(r, g, b))
                .add_modifier(Modifier::BOLD),
        ));
    }
    Line::from(spans)
}

/// Linear interpolation from cyan `(80, 200, 255)` at column 0 to
/// magenta `(220, 100, 220)` at the rightmost column. Slightly
/// muted endpoints — closer to the soft pastel gradient that reads
/// well on most terminal backgrounds.
pub fn gradient_at(col: usize, total: usize) -> (u8, u8, u8) {
    let t = if total <= 1 {
        0.0
    } else {
        col as f32 / (total - 1) as f32
    };
    let r = lerp(80.0, 220.0, t) as u8;
    let g = lerp(200.0, 100.0, t) as u8;
    let b = lerp(255.0, 220.0, t) as u8;
    (r, g, b)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn sample_stats() -> Vec<String> {
        vec![
            "plans:32".into(),
            "tasks:99".into(),
            "agents:5".into(),
            "prs:7".into(),
            "v0.3.2".into(),
        ]
    }

    #[test]
    fn header_height_picks_banner_when_wide() {
        assert_eq!(header_height(120), BANNER_HEIGHT);
        assert_eq!(header_height(STACKED_MIN_WIDTH), BANNER_HEIGHT);
    }

    #[test]
    fn header_height_falls_back_to_compact_when_narrow() {
        assert_eq!(header_height(40), COMPACT_HEIGHT);
        assert_eq!(header_height(STACKED_MIN_WIDTH - 1), COMPACT_HEIGHT);
    }

    #[test]
    fn gradient_endpoints_match_design_constants() {
        assert_eq!(gradient_at(0, 50), (80, 200, 255));
        assert_eq!(gradient_at(49, 50), (220, 100, 220));
    }

    #[test]
    fn render_side_by_side_writes_banner_and_right_aligned_stats() {
        let backend = TestBackend::new(120, BANNER_HEIGHT);
        let mut term = Terminal::new(backend).unwrap();
        let stats = sample_stats();
        term.draw(|f| render(f, f.area(), &stats)).unwrap();
        let buf = term.backend().buffer();
        let dump = buf.content().iter().map(|c| c.symbol()).collect::<String>();
        assert!(dump.contains("█"), "ANSI shadow blocks missing: {dump:?}");
        assert!(dump.contains("plans:32"), "stats first line missing");
        assert!(dump.contains("v0.3.2"), "stats version line missing");
    }

    #[test]
    fn render_stacked_writes_banner_above_inline_stats() {
        let backend = TestBackend::new(80, BANNER_HEIGHT);
        let mut term = Terminal::new(backend).unwrap();
        let stats = sample_stats();
        term.draw(|f| render(f, f.area(), &stats)).unwrap();
        let buf = term.backend().buffer();
        let dump = buf.content().iter().map(|c| c.symbol()).collect::<String>();
        assert!(dump.contains("█"));
        assert!(dump.contains("plans:32"));
        assert!(dump.contains("·"), "stacked stats line uses · separator");
    }

    #[test]
    fn render_compact_used_on_narrow_terms() {
        let backend = TestBackend::new(40, 1);
        let mut term = Terminal::new(backend).unwrap();
        let stats = sample_stats();
        term.draw(|f| render(f, f.area(), &stats)).unwrap();
        let buf = term.backend().buffer();
        let dump = buf.content().iter().map(|c| c.symbol()).collect::<String>();
        assert!(
            dump.contains("CONVERGIO"),
            "compact wordmark missing: {dump:?}"
        );
    }
}
