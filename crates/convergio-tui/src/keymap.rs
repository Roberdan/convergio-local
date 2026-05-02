//! Key dispatch.
//!
//! Translates a [`crossterm::event::KeyEvent`] into a high-level
//! [`Action`] that the event loop in `lib.rs` consumes. Centralising
//! the mapping keeps `lib.rs` short and makes per-key behaviour
//! testable without spinning a terminal.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// High-level action produced by a key press.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Quit the dashboard and restore the terminal.
    Quit,
    /// Skip the tick wait and refresh now.
    RefreshNow,
    /// Move focus to the next pane in tab order.
    PaneNext,
    /// Move focus to the previous pane.
    PanePrev,
    /// Cursor down within the focused pane.
    RowDown,
    /// Cursor up within the focused pane.
    RowUp,
    /// Key was bound to no action — caller ignores.
    Noop,
}

/// Default key binding. Stateless. Cloneable.
#[derive(Debug, Default, Clone, Copy)]
pub struct KeyMap;

impl KeyMap {
    /// Map a key event to an [`Action`].
    pub fn translate(&self, key: KeyEvent) -> Action {
        // Ctrl+C is always quit, regardless of focus.
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Action::Quit;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
            KeyCode::Char('r') => Action::RefreshNow,
            KeyCode::Tab => Action::PaneNext,
            KeyCode::BackTab => Action::PanePrev,
            KeyCode::Char('j') | KeyCode::Down => Action::RowDown,
            KeyCode::Char('k') | KeyCode::Up => Action::RowUp,
            _ => Action::Noop,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode::*;
    use crossterm::event::{KeyEventKind, KeyEventState};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn q_and_esc_quit() {
        let km = KeyMap;
        assert_eq!(km.translate(key(Char('q'))), Action::Quit);
        assert_eq!(km.translate(key(Esc)), Action::Quit);
    }

    #[test]
    fn ctrl_c_quits_even_if_inside_input() {
        let km = KeyMap;
        assert_eq!(km.translate(ctrl(Char('c'))), Action::Quit);
    }

    #[test]
    fn r_refreshes() {
        let km = KeyMap;
        assert_eq!(km.translate(key(Char('r'))), Action::RefreshNow);
    }

    #[test]
    fn tab_moves_pane_forward_and_shift_tab_back() {
        let km = KeyMap;
        assert_eq!(km.translate(key(Tab)), Action::PaneNext);
        assert_eq!(km.translate(key(BackTab)), Action::PanePrev);
    }

    #[test]
    fn j_k_arrows_move_rows() {
        let km = KeyMap;
        assert_eq!(km.translate(key(Char('j'))), Action::RowDown);
        assert_eq!(km.translate(key(Down)), Action::RowDown);
        assert_eq!(km.translate(key(Char('k'))), Action::RowUp);
        assert_eq!(km.translate(key(Up)), Action::RowUp);
    }

    #[test]
    fn unbound_keys_are_noop() {
        let km = KeyMap;
        assert_eq!(km.translate(key(Char('x'))), Action::Noop);
    }
}
