//! Animated gradient banner for the dashboard header.
//!
//! Renders a stylised "CONVERGIO" wordmark with a cyan→magenta
//! gradient. Each character cell carries an RGB foreground colour
//! interpolated from its column position. Terminals without
//! true-colour fall back to ratatui's nearest 256-colour mapping.
//!
//! Falls back to a single-line bold banner when the available width
//! is smaller than the wordmark — the dashboard must remain usable
//! on 80×24 terminals (CONSTITUTION P3).

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

/// 5-line block-character wordmark. 53 columns wide.
const WORDMARK: &[&str] = &[
    "█▀▀ █▀█ █▄ █ █ █ █▀▀ █▀▄ █▀▀ █ █▀█",
    "█   █ █ █ ██ █ █ █▀  █▀▄ █ █ █ █ █",
    "█▄▄ █▄█ █  █  ▀▀  █▄▄ █ █ █▄█ █ █▄█",
];

/// Width below which we skip the multi-line banner and render a
/// single-line bold title instead.
const MIN_BANNER_WIDTH: u16 = 60;

/// Height the banner consumes when shown (3 wordmark + 1 stats).
pub const BANNER_HEIGHT: u16 = 4;

/// Height of the compact (single-line) header.
pub const COMPACT_HEIGHT: u16 = 1;

/// Returns the height the header should reserve given `width`.
pub fn header_height(width: u16) -> u16 {
    if width >= MIN_BANNER_WIDTH {
        BANNER_HEIGHT
    } else {
        COMPACT_HEIGHT
    }
}

/// Render the header into `area`, automatically picking the banner
/// or compact form. `subtitle` is the second line shown under the
/// banner (in compact mode, replaces the banner).
pub fn render(f: &mut Frame, area: Rect, subtitle: &str) {
    if area.width >= MIN_BANNER_WIDTH && area.height >= BANNER_HEIGHT {
        render_banner(f, area, subtitle);
    } else {
        render_compact(f, area, subtitle);
    }
}

fn render_banner(f: &mut Frame, area: Rect, subtitle: &str) {
    let total_cols = WORDMARK
        .iter()
        .map(|r| r.chars().count())
        .max()
        .unwrap_or(1);
    let mut lines: Vec<Line> = WORDMARK
        .iter()
        .map(|row| line_with_gradient(row, total_cols))
        .collect();
    lines.push(Line::from(Span::styled(
        subtitle.to_string(),
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )));
    let p = Paragraph::new(lines);
    f.render_widget(p, area);
}

fn render_compact(f: &mut Frame, area: Rect, subtitle: &str) {
    let line = Line::from(vec![
        Span::styled(
            "▌C◆O◆N◆V◆E◆R◆G◆I◆O▐ ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(subtitle.to_string(), Style::default().fg(Color::DarkGray)),
    ]);
    f.render_widget(Paragraph::new(line), area);
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
/// magenta `(220, 100, 220)` at the rightmost column. Pure cyan and
/// pure magenta are too saturated for most terminals, so we use
/// slightly muted endpoints — closer to the soft pastel gradient
/// people associate with modern brand wordmarks.
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

    #[test]
    fn header_height_picks_banner_for_wide_terms() {
        assert_eq!(header_height(120), BANNER_HEIGHT);
        assert_eq!(header_height(60), BANNER_HEIGHT);
    }

    #[test]
    fn header_height_falls_back_to_compact_for_narrow_terms() {
        assert_eq!(header_height(40), COMPACT_HEIGHT);
        assert_eq!(header_height(0), COMPACT_HEIGHT);
    }

    #[test]
    fn gradient_endpoints_match_design_constants() {
        let (r0, g0, b0) = gradient_at(0, 50);
        assert_eq!((r0, g0, b0), (80, 200, 255));
        let (rn, gn, bn) = gradient_at(49, 50);
        assert_eq!((rn, gn, bn), (220, 100, 220));
    }

    #[test]
    fn gradient_is_monotonic_in_red_channel() {
        let mut prev = 0u8;
        for c in 0..50 {
            let (r, _, _) = gradient_at(c, 50);
            assert!(r >= prev, "red should grow column {c}: {prev} -> {r}");
            prev = r;
        }
    }

    #[test]
    fn render_banner_writes_convergio_glyphs() {
        let backend = TestBackend::new(80, 6);
        let mut term = Terminal::new(backend).unwrap();
        term.draw(|f| render(f, f.area(), "v0.3.2 · plans 5"))
            .unwrap();
        let buf = term.backend().buffer();
        let dump = buf.content().iter().map(|c| c.symbol()).collect::<String>();
        assert!(dump.contains("█"), "block glyphs should appear: {dump:?}");
        assert!(
            dump.contains("v0.3.2"),
            "subtitle should appear under banner: {dump:?}"
        );
    }

    #[test]
    fn render_compact_used_on_narrow_term() {
        let backend = TestBackend::new(40, 1);
        let mut term = Terminal::new(backend).unwrap();
        term.draw(|f| render(f, f.area(), "v0.3.2")).unwrap();
        let buf = term.backend().buffer();
        let dump = buf.content().iter().map(|c| c.symbol()).collect::<String>();
        assert!(
            dump.contains("CONVERGIO") || dump.contains("C◆O"),
            "compact wordmark missing: {dump:?}"
        );
    }
}
