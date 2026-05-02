//! `cvg dash` — open the TUI dashboard.
//!
//! Tiny shim that hands control to the [`convergio_tui`] crate. The
//! crate boundary is intentional (ADR-0029): keeping ratatui and
//! crossterm out of the daemon and out of every cvg subcommand keeps
//! their dependency tree off the hot CLI path. Read
//! [crate-level AGENTS.md](../../convergio-tui/AGENTS.md) before
//! changing the dashboard surface.

use anyhow::Result;

/// Entry point for `cvg dash`. Resolves the workspace's GitHub slug
/// (best-effort) so the PRs pane is scoped to this repository
/// regardless of cwd. Forwards everything to
/// [`convergio_tui::run`], which owns terminal setup/teardown.
pub async fn run(daemon_url: &str, tick_secs: u64) -> Result<()> {
    let slug = super::update_repo_root::resolve()
        .ok()
        .and_then(|root| super::update_repo_root::github_slug(&root));
    convergio_tui::run(daemon_url, tick_secs, slug).await
}
