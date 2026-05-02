---
id: 0029
status: accepted
date: 2026-05-02
topics: [cli, tui, dashboard, ui, urbanism]
related_adrs: [0009, 0015, 0018]
touches_crates: [convergio-cli, convergio-tui]
last_validated: 2026-05-02
---

# 0029. TUI dashboard lives in its own crate (`convergio-tui`)

- Status: accepted
- Date: 2026-05-02
- Deciders: Roberdan
- Tags: cli, tui, dashboard, ui

## Context and Problem Statement

Operators of a local Convergio daemon need a console that summarises
**plans, active tasks, agents, and open PRs** in one view, refreshing
on a tick. `cvg status` produces a snapshot today, and `cvg session
resume` produces a cold-start brief — both are great for a single
moment in time, neither is a console you leave open in a tmux pane.

The implementation choice was: where does the TUI live?

1. Inside `convergio-cli` — keep it next to every other subcommand.
2. Inside `convergio-server` — it speaks the same wire format.
3. In a separate crate `convergio-tui` consumed by `convergio-cli`.

`convergio-cli` was already at **7 516 LOC** with eight files within
50 lines of the **300-line cap** ([CONSTITUTION § 13](../../CONSTITUTION.md)).
`ratatui + crossterm` add a non-trivial transitive tree (`unicode-width`,
`signal-hook`, `mio`, ...) that has no business in the daemon. Splitting
the TUI into its own crate keeps each boundary honest:

- `convergio-cli` stays a pure HTTP client.
- `convergio-server` (the daemon) stays free of UI dependencies.
- `convergio-tui` owns terminal handling, layout, and rendering.

The same pattern was applied to `convergio-graph` for the same reason
(big dependency footprint, distinct boundary, ADR-0014).

## Decision Drivers

- **Boundary clarity (CONSTITUTION § Crates).** A new presentation
  surface is a new crate, not a new module in an existing one.
- **300-line cap respected.** Spreading the dashboard across one crate
  with a per-pane file keeps every Rust file under the cap.
- **Daemon stays light.** The daemon binary should not pull in
  ratatui's tree because it never renders anything.
- **No business logic in the dashboard.** Every number on screen
  comes 1:1 from a daemon HTTP response. The dashboard is a viewer,
  not a thinker.
- **Read-only contract.** The MVP issues only `GET` + `gh pr list`.
  This keeps the threat model trivial: the TUI cannot accidentally
  mutate plan state or write evidence.
- **Urbanism (ADR-0018).** A dashboard is a piazza — a place humans
  read the city. It belongs in its own building, not bolted to the
  client crate.

## Considered Options

1. **Module inside `convergio-cli`.** Rejected: pushes cli/Cargo.toml
   over the dep budget the crate has been holding, and the crate
   already runs hot on the 300-line cap.
2. **Module inside `convergio-server`.** Rejected: the daemon would
   pull a TUI dep tree it never uses; it would also blur the leash
   metaphor (the daemon is the gate, not the operator's console).
3. **Separate crate `convergio-tui` (chosen).** Adds 1 new workspace
   member, one new top-level subcommand (`cvg dash`), and isolates
   ratatui/crossterm from every other surface.
4. **Web UI on a port.** Rejected for the MVP — a TUI runs over ssh
   without port-forward, and the operator already lives in a
   terminal. A web option may follow in a separate ADR.

## Decision Outcome

**(3) — separate crate `convergio-tui`.** Consumed only by
`convergio-cli`'s new `dash` subcommand:

```
cvg dash [--tick-secs N]
```

`convergio-tui` exposes one entry point:

```rust
pub async fn run(daemon_url: &str, tick_secs: u64) -> Result<()>
```

`convergio-cli`'s `commands/dash.rs` is a 15-line shim: it forwards
arguments and returns. No layout work, no rendering, no terminal
setup — those all live in `convergio-tui`.

### Boundary contract

The crate's [`AGENTS.md`](../../crates/convergio-tui/AGENTS.md) makes
this explicit:

- **Read-only on the daemon.** `GET` only. State-changing commands
  belong in dedicated `cvg` subcommands. A future interactive
  evolution (`:validate <plan>`) lives behind a follow-up ADR.
- **No business logic.** Every figure rendered must be a 1:1 view of
  a daemon response.
- **No imports of `convergio-cli` / `convergio-server` / SQLite.**
- **Each pane in its own file** to honour the 300-line cap.

### Distribution

`convergio-tui` is a library crate. The `cvg` binary statically links
it. There is no separate `cvg-dash` executable. The same
`scripts/install-local.sh` that installs the daemon, the CLI, and the
MCP bridge also installs the dashboard as part of `cvg`.

## Consequences

- **Positive.** A 4-pane console is one command (`cvg dash`) away.
  Operators can leave a dedicated terminal pane open and watch state
  change in real time.
- **Positive.** The daemon stays free of TUI deps. `cargo install
  --path crates/convergio-server` does not pull ratatui.
- **Positive.** The crate is the natural home for follow-ups — an
  interactive `:command` mode, a separate `cvg follow` event log, a
  detail-drill pane — without touching `convergio-cli`.
- **Negative.** One more workspace member to maintain. Mitigated:
  the crate is small and the contract is narrow; there are no
  schema/migration obligations.
- **Negative.** The 300-line cap forces a per-pane file split that
  feels slightly ceremonial. Mitigated: each file owns a clean
  responsibility (one pane = one file), which makes adding a new
  pane mechanical.

## Validation

- `cargo fmt --all -- --check` clean.
- `RUSTFLAGS=-Dwarnings cargo clippy --workspace --all-targets
  -- -D warnings` clean.
- `cargo test -p convergio-tui --lib` — 32 unit tests pass (renderer
  snapshots via `ratatui::backend::TestBackend`, plus state and
  client helpers).
- `cvg dash --help` prints the documented usage.
- File sizes: every `*.rs` under `crates/convergio-tui/src/` is under
  the 300-line cap.

## Out of scope

- Interactive command surface (`:validate`, `:claim`, `:retire`).
  Lives in a follow-up ADR if and when needed.
- Detail-drill (`Enter` opens a per-row detail) — designed for, but
  not implemented in, the MVP.
- A web dashboard. Would need a separate ADR; the TUI satisfies the
  "ssh-friendly console" use case.
- i18n of pure layout labels. Status names already flow through
  `convergio-i18n`; pure layout strings (`Plans`, `Tasks`, …) start
  English-only and are migrated later if a non-English operator
  reports it as a barrier.
