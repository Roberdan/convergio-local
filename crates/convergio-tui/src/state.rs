//! Aggregate state for the dashboard.
//!
//! [`AppState`] owns the four datasets the panes render plus the
//! focus + scroll position for each pane. Refreshes are issued by
//! [`AppState::refresh`] which delegates to [`crate::client::Client`].

use crate::client::{Client, Plan, PrSummary, RegistryAgent, TaskSummary};
pub use crate::mode::{AppMode, DetailTarget};

/// The four panes rendered by the dashboard, in tab order.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    /// Plans (top-left). Default focus on startup.
    #[default]
    Plans,
    /// Active tasks across plans (top-right).
    Tasks,
    /// Registered agents (bottom-left).
    Agents,
    /// Open pull requests (bottom-right).
    Prs,
}

impl Pane {
    /// All panes in display order.
    pub const ALL: [Pane; 4] = [Pane::Plans, Pane::Tasks, Pane::Agents, Pane::Prs];

    /// Short label rendered as the pane title.
    pub fn label(&self) -> &'static str {
        match self {
            Pane::Plans => "Plans",
            Pane::Tasks => "Active tasks",
            Pane::Agents => "Agents",
            Pane::Prs => "PRs",
        }
    }
}

/// Per-pane scroll offset.
#[derive(Debug, Default, Clone, Copy)]
pub struct Cursor {
    /// First row index visible in the pane.
    pub offset: usize,
    /// Selected row, relative to all rows in the pane (NOT to offset).
    pub selected: usize,
}

impl Cursor {
    /// Move the selection one row down, capped at `max_idx`. Adjusts
    /// the offset only enough to keep the selection visible.
    pub fn down(&mut self, max_idx: usize, page: usize) {
        if max_idx == 0 {
            return;
        }
        self.selected = (self.selected + 1).min(max_idx - 1);
        if self.selected >= self.offset + page {
            self.offset = self.selected + 1 - page;
        }
    }

    /// Move the selection one row up.
    pub fn up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
        if self.selected < self.offset {
            self.offset = self.selected;
        }
    }
}

/// Connection / refresh status surfaced in the footer.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Connection {
    /// First refresh has not completed yet.
    #[default]
    Initial,
    /// Last refresh succeeded.
    Connected,
    /// Last refresh failed (network or 4xx/5xx).
    Disconnected,
}

/// Aggregate dashboard state.
///
/// `Default` produces an empty state (no plans, etc.). Call
/// [`AppState::refresh`] to populate.
#[derive(Debug, Default)]
pub struct AppState {
    /// Plans returned by the daemon.
    pub plans: Vec<Plan>,
    /// Active tasks across plans (status `in_progress` or `submitted`).
    pub tasks: Vec<TaskSummary>,
    /// Registered agents.
    pub agents: Vec<RegistryAgent>,
    /// Open pull requests via `gh pr list`.
    pub prs: Vec<PrSummary>,
    /// Audit chain ok/not.
    pub audit_ok: Option<bool>,
    /// Daemon version reported by `GET /v1/health`. `None` until the
    /// first successful refresh.
    pub daemon_version: Option<String>,
    /// Connection state for the footer.
    pub connection: Connection,
    /// UTC timestamp of the last successful refresh.
    pub last_refresh: Option<chrono::DateTime<chrono::Utc>>,
    /// Currently focused pane.
    pub focus: Pane,
    /// Per-pane cursor.
    pub cursor: PaneCursors,
    /// Active UI mode (Overview vs drill-down).
    pub mode: AppMode,
    /// Cached task list for the plan currently being drilled into.
    /// Populated by [`AppState::enter_detail`] for `Plan` targets so
    /// the detail panel shows every task (not only the active subset
    /// that the overview pane carries).
    pub detail_tasks: Vec<TaskSummary>,
}

/// Cursors for the four panes, addressable by [`Pane`].
#[derive(Debug, Default, Clone, Copy)]
pub struct PaneCursors {
    /// Cursor for the Plans pane.
    pub plans: Cursor,
    /// Cursor for the Active Tasks pane.
    pub tasks: Cursor,
    /// Cursor for the Agents pane.
    pub agents: Cursor,
    /// Cursor for the PRs pane.
    pub prs: Cursor,
}

/// Compile-time version of the `cvg` binary embedding this dashboard.
/// Compared against the live daemon version to surface drift.
pub const BINARY_VERSION: &str = env!("CARGO_PKG_VERSION");

/// `Some(daemon)` when the daemon and the binary report different
/// versions, `None` when they match or the daemon is unreachable.
pub fn version_drift(daemon: Option<&str>) -> Option<String> {
    let d = daemon?;
    if d == BINARY_VERSION {
        None
    } else {
        Some(d.to_string())
    }
}

impl AppState {
    /// Refresh every dataset. Failures roll up into
    /// [`Connection::Disconnected`] and leave the previous data in
    /// place — the dashboard never blanks itself on a transient
    /// network error.
    pub async fn refresh(&mut self, client: &Client) {
        let snapshot = client.snapshot().await;
        match snapshot {
            Ok(s) => {
                self.plans = s.plans;
                self.tasks = s.tasks;
                self.agents = s.agents;
                self.prs = s.prs;
                self.audit_ok = s.audit_ok;
                self.daemon_version = s.daemon_version;
                self.connection = Connection::Connected;
                self.last_refresh = Some(chrono::Utc::now());
            }
            Err(_) => {
                self.connection = Connection::Disconnected;
            }
        }
    }

    /// Move focus to the next pane in tab order.
    pub fn focus_next(&mut self) {
        let idx = Pane::ALL.iter().position(|p| *p == self.focus).unwrap_or(0);
        self.focus = Pane::ALL[(idx + 1) % Pane::ALL.len()];
    }

    /// Move focus to the previous pane.
    pub fn focus_prev(&mut self) {
        let idx = Pane::ALL.iter().position(|p| *p == self.focus).unwrap_or(0);
        self.focus = Pane::ALL[(idx + Pane::ALL.len() - 1) % Pane::ALL.len()];
    }

    /// Cursor down within the focused pane.
    pub fn row_down(&mut self) {
        let (cursor, len) = self.focused_cursor_and_len_mut();
        cursor.down(len, 8);
    }

    /// Cursor up within the focused pane.
    pub fn row_up(&mut self) {
        let (cursor, _) = self.focused_cursor_and_len_mut();
        cursor.up();
    }

    /// Build the [`DetailTarget`] for the row currently selected in
    /// the focused pane, if any. Returns `None` when the pane is empty
    /// or the cursor points past the end (refresh race).
    pub fn drill_target(&self) -> Option<DetailTarget> {
        match self.focus {
            Pane::Plans => self
                .plans
                .get(self.cursor.plans.selected)
                .map(|p| DetailTarget::Plan {
                    id: p.id.clone(),
                    title: p.title.clone(),
                }),
            Pane::Tasks => self
                .tasks
                .get(self.cursor.tasks.selected)
                .map(|t| DetailTarget::Task {
                    id: t.id.clone(),
                    plan_id: t.plan_id.clone(),
                    title: t.title.clone(),
                }),
            Pane::Agents => self
                .agents
                .get(self.cursor.agents.selected)
                .map(|a| DetailTarget::Agent { id: a.id.clone() }),
            Pane::Prs => self
                .prs
                .get(self.cursor.prs.selected)
                .map(|p| DetailTarget::Pr {
                    number: p.number,
                    title: p.title.clone(),
                }),
        }
    }

    /// Enter detail mode against a target.
    ///
    /// For [`DetailTarget::Plan`], this also fetches the full task list
    /// for the plan into [`AppState::detail_tasks`]. The fetch is the
    /// one place the overview's "active-only" filter is widened.
    pub async fn enter_detail(&mut self, client: &Client, target: DetailTarget) {
        if let DetailTarget::Plan { id, .. } = &target {
            self.detail_tasks = client.fetch_plan_tasks(id).await.unwrap_or_default();
        } else {
            self.detail_tasks.clear();
        }
        self.mode = AppMode::Detail(target);
    }

    /// Leave detail mode and return to the 4-pane overview.
    pub fn back_to_overview(&mut self) {
        self.mode = AppMode::Overview;
        self.detail_tasks.clear();
    }

    fn focused_cursor_and_len_mut(&mut self) -> (&mut Cursor, usize) {
        match self.focus {
            Pane::Plans => (&mut self.cursor.plans, self.plans.len()),
            Pane::Tasks => (&mut self.cursor.tasks, self.tasks.len()),
            Pane::Agents => (&mut self.cursor.agents, self.agents.len()),
            Pane::Prs => (&mut self.cursor.prs, self.prs.len()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pane_all_covers_four_panes() {
        assert_eq!(Pane::ALL.len(), 4);
    }

    #[test]
    fn focus_cycles_forward_and_backward() {
        let mut s = AppState::default();
        assert_eq!(s.focus, Pane::Plans);
        s.focus_next();
        s.focus_next();
        s.focus_next();
        s.focus_next();
        assert_eq!(s.focus, Pane::Plans, "wraps after 4 hops");
        s.focus_prev();
        assert_eq!(s.focus, Pane::Prs);
    }

    #[test]
    fn cursor_down_caps_at_last_row_and_noop_on_empty() {
        let mut c = Cursor::default();
        for _ in 0..4 {
            c.down(3, 2);
        }
        assert_eq!(c.selected, 2);
        let mut c2 = Cursor::default();
        c2.down(0, 5);
        assert_eq!((c2.selected, c2.offset), (0, 0));
    }

    #[test]
    fn cursor_up_does_not_underflow() {
        let mut c = Cursor::default();
        c.up();
        assert_eq!(c.selected, 0);
    }
}
