# AGENTS.md

**Cross-vendor agent instructions for Convergio v3.** This file is read by
Codex (`AGENTS.md`), Cursor (`.cursor/rules/`), GitHub Copilot
(`.github/copilot-instructions.md`), and Claude Code (`CLAUDE.md`).
The other three are symlinks to this file. **Edit only this one.**

If you are a human, read [README.md](./README.md) first.

---

## Project in one paragraph

Convergio is a Rust HTTP daemon that provides a **durability layer** for AI
agent workflows: persistent state for plans/tasks/evidence, a
hash-chained audit log, an inter-agent message bus, and supervision of
long-running agent processes. It runs in two modes (personal SQLite,
team Postgres) from a single binary. It is **not** an agent framework —
LangGraph, CrewAI, Claude Code skills are clients, not competitors.

See [docs/spec/v3-durability-layer.md](./docs/spec/v3-durability-layer.md)
for the full spec, [ARCHITECTURE.md](./ARCHITECTURE.md) for the layer
diagram, [CONSTITUTION.md](./CONSTITUTION.md) for non-negotiable rules.

## Stack

- **Language**: Rust (stable, pinned via `rust-toolchain.toml`)
- **HTTP**: `axum 0.7` — path params use `:id` (NOT `{id}`)
- **DB**: `sqlx` with `sqlite` (personal) + `postgres` (team) features
- **Async**: `tokio` (multi-thread runtime)
- **CLI**: `clap` derive
- **Logging**: `tracing` + `tracing-subscriber` (json in team mode)
- **Hashing (audit chain)**: `sha2`
- **IDs**: `uuid` v4

## Repo layout

```
convergioV3/
├── AGENTS.md             ← you are here (single source of truth)
├── CLAUDE.md → AGENTS.md
├── .github/copilot-instructions.md → ../AGENTS.md
├── .cursor/rules/        ← points back here
├── README.md             ← human entry point
├── ARCHITECTURE.md       ← 4-layer diagram, request lifecycle
├── CONSTITUTION.md       ← 11 non-negotiable rules
├── ROADMAP.md            ← 8-week MVP plan
├── CHANGELOG.md
├── LICENSE               ← Apache 2.0
├── SECURITY.md
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── Cargo.toml            ← workspace
├── Cargo.lock            ← committed (we ship a binary)
├── rust-toolchain.toml
├── .cargo/config.toml
├── .mcp.json
├── lefthook.yml
├── commitlint.config.js
├── crates/
│   ├── convergio-db/             ← Layer 0 — sqlx pool + migrations
│   ├── convergio-durability/     ← Layer 1 — plans/tasks/evidence/audit/gates
│   ├── convergio-bus/            ← Layer 2 — agent message bus (skeleton)
│   ├── convergio-lifecycle/      ← Layer 3 — agent spawn/supervise (skeleton)
│   ├── convergio-server/         ← shell — axum routing
│   ├── convergio-cli/            ← `cvg` binary, pure HTTP client
│   ├── convergio-planner/        ← Layer 4 — solve (skeleton)
│   ├── convergio-thor/           ← Layer 4 — validator (skeleton)
│   ├── convergio-executor/       ← Layer 4 — task dispatcher (skeleton)
│   └── convergio-worktree/       ← Layer 4 — git worktree integration (skeleton)
├── docs/
│   ├── adr/                ← architecture decision records (MADR)
│   ├── spec/               ← specs and design docs
│   └── plans/              ← active YAML plans
```

E2E tests live under each crate's own `tests/` directory (Cargo
convention). The cross-crate end-to-end test that boots the server
in-process lives in `crates/convergio-server/tests/`.

## Build & run

```bash
# build everything
cargo build --workspace

# run the daemon (personal mode by default)
cargo run -p convergio-server
# → SQLite at ~/.convergio/state.db, listens on 127.0.0.1:8420

# CLI
cargo run -p convergio-cli -- health
cargo run -p convergio-cli -- plan create "my plan"
```

## Test

The authoritative pre-push check (matches CI):

```bash
cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" cargo clippy --workspace --all-targets -- -D warnings
RUSTFLAGS="-Dwarnings" cargo test --workspace
```

Faster targeted runs:

```bash
cargo test -p convergio-durability                      # one crate
cargo test -p convergio-durability audit_chain          # one test
cargo check -p convergio-server                         # type-check only
```

E2E tests live in `tests/` at the workspace root and boot the server
in-process. They use a tempdir SQLite by default — no manual setup.

## Code style (hook-enforced and CI-enforced)

| Rule | Enforced by |
|------|-------------|
| Max **300 lines/file** for `*.rs` | lefthook `pre-commit` + CI |
| `cargo fmt` clean | lefthook + CI |
| `cargo clippy -D warnings` clean | CI |
| Conventional commits with crate scope | commitlint via lefthook `commit-msg` |
| No `unwrap()` / `expect()` in production code (tests fine) | clippy lint, manual review |
| Every `pub` item has `///` doc comment | clippy `missing_docs` lint per crate |
| `//!` crate-level doc at top of every `lib.rs` and `main.rs` | manual review |

Commit scope must be a known crate name or one of `docs|ci|chore|repo|deps`.
Examples: `feat(durability): add audit hash chain`, `fix(server): handle 409 on gate refusal`.

## Where to put new code

| You want to... | Put it in... |
|----------------|--------------|
| Add a new HTTP endpoint | `crates/convergio-server/src/routes/` (one file per resource) |
| Add a new gate | `crates/convergio-durability/src/gates/` (one file per gate) |
| Add a CLI subcommand | `crates/convergio-cli/src/commands/` (one file per subcommand) |
| Add a new DB table | New migration in `crates/convergio-durability/migrations/` AND module in `src/store/` |
| Add a planner heuristic | `crates/convergio-planner/src/` |
| Document a design decision | `docs/adr/NNNN-short-title.md` (next free number, MADR template) |
| Track ongoing work | `docs/plans/<plan>.yaml` (or actually use the daemon once it works) |

If unsure, **read the relevant crate's `src/lib.rs` `//!` doc first** —
it should tell you the entry point.

## Do-not-touch

- `Cargo.lock` — automatic only via `cargo update`
- Generated migration files — never hand-edit; create a new migration
- `LICENSE` — Apache 2.0, immutable
- Anything in `target/`

## MCP tools available

Project-level `.mcp.json` declares the MCP servers active in this repo.
For the **daemon's own tools** (`cvg_*` family), prefer them over raw
`curl` once the daemon is running locally.

The most useful tools (once Layer 1 is wired):

- `cvg_create_plan`, `cvg_get_plan`, `cvg_list_plans`
- `cvg_record_evidence`, `cvg_update_task`
- `cvg_health`, `cvg_doctor_run`

## Pull requests

Every PR body MUST contain these 5 H2 sections (CI-enforced via
`.github/pull_request_template.md`):

```markdown
## Problem
## Why
## What changed
## Validation
## Impact
```

## Background loops in the daemon

There is exactly **one** background loop in Layer 1:

- `Reaper` (60s) — releases tasks whose agent's heartbeat is stale.

Layer 4 may add executor/planner loops. **Do not document loops you have
not actually implemented.** (We had this exact lie in v2 docs for
months — not again.)

## When in doubt

1. Read the relevant `crates/<name>/src/lib.rs` `//!` block.
2. Read the relevant ADR in `docs/adr/`.
3. Read [CONSTITUTION.md](./CONSTITUTION.md).
4. Ask the human. Do not invent.
