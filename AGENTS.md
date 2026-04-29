# AGENTS.md

**Cross-vendor agent instructions for Convergio v3.** This file is read by
Codex (`AGENTS.md`), Cursor (`.cursor/rules/`), GitHub Copilot
(`.github/copilot-instructions.md`), and Claude Code (`CLAUDE.md`).
The other three are symlinks to this file. **Edit only this one.**

If you are a human, read [README.md](./README.md) first.

---

## Project in one paragraph

Convergio is the **leash** for AI agents. It is a Rust HTTP daemon
that refuses the agent's work when the work does not meet five
non-negotiable principles (CONSTITUTION § Sacred principles):

1. **Zero tolerance** for technical debt, errors, warnings — any programming language
2. **Security first**, including LLM-specific threats
3. **Accessibility first**, both in agent output and in our own CLI
4. **No scaffolding only** — every feature must be fully wired
5. **Internationalization first** — Italian + English day one, no hardcoded user-facing English

Principles are enforced server-side via the gate pipeline. The agent
attaches evidence of work done; gates scan that evidence and refuse
`submitted`/`done` transitions with HTTP 409 if any rule is violated.
Underneath sit a hash-chained audit log (so nothing can be silently
rewritten), a reaper (so agent death does not lose state), an
agent-to-agent message bus, and process supervision.

Single-user, local-first, SQLite-only. Drop-in under any local
orchestrator or agent runner (LangGraph, CrewAI, Claude Code skills,
shell scripts, your own Python).

See [ARCHITECTURE.md](./ARCHITECTURE.md) for layer diagrams,
[CONSTITUTION.md](./CONSTITUTION.md) for the non-negotiable rules,
[docs/adr/0004-three-sacred-principles.md](./docs/adr/0004-three-sacred-principles.md)
for the rationale, [docs/spec/v3-durability-layer.md](./docs/spec/v3-durability-layer.md)
for the original spec.

## Stack

- **Language**: Rust (stable, pinned via `rust-toolchain.toml`)
- **HTTP**: `axum 0.7` — path params use `:id` (NOT `{id}`)
- **DB**: `sqlx` with SQLite only
- **Async**: `tokio` (multi-thread runtime)
- **CLI**: `clap` derive
- **Logging**: `tracing` + `tracing-subscriber`
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
│   ├── convergio-bus/            ← Layer 2 — agent message bus
│   ├── convergio-lifecycle/      ← Layer 3 — agent spawn/supervise
│   ├── convergio-server/         ← shell — axum routing
│   ├── convergio-cli/            ← `cvg` binary, pure HTTP client
│   ├── convergio-planner/        ← Layer 4 — solve
│   ├── convergio-thor/           ← Layer 4 — validator
│   └── convergio-executor/       ← Layer 4 — task dispatcher
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

# run the local daemon
cargo run -p convergio-server -- start
# → SQLite at ~/.convergio/v3/state.db, listens on 127.0.0.1:8420

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

Test suite layout (165 tests as of local-first scope):

| Target | Tests |
|--------|-------|
| `convergio-db` (unit) | 3 |
| `convergio-durability` (unit) | 6 |
| `convergio-durability/tests/audit_tamper.rs` | 7 — proves ADR-0002 |
| `convergio-durability/tests/gates.rs` | 7 |
| `convergio-durability/tests/no_debt_gate.rs` | 8 — proves P1 |
| `convergio-durability/tests/no_debt_gate_multilang.rs` | 16 — covers 7 languages |
| `convergio-durability/tests/zero_warnings_gate.rs` | 8 — proves P1 build/lint signal |
| `convergio-durability/tests/reaper.rs` | 3 |
| `convergio-bus/tests/lifecycle.rs` | 6 |
| `convergio-lifecycle/tests/spawn.rs` | 4 |
| `convergio-lifecycle/tests/watcher.rs` | 3 |
| `convergio-planner/tests/solve.rs` | 5 |
| `convergio-thor/tests/validate.rs` | 4 |
| `convergio-executor/tests/dispatch.rs` | 4 |
| `convergio-cli/tests/cli_smoke.rs` | 17 |
| `convergio-server/tests/e2e_durability.rs` | 1 |
| `convergio-server/tests/e2e_bus.rs` | 2 |
| `convergio-server/tests/e2e_agents.rs` | 2 |
| `convergio-server/tests/e2e_audit.rs` | 3 |
| `convergio-server/tests/e2e_full_stack.rs` | 1 |
| `convergio-server/tests/e2e_quickstart.rs` | 2 |
| `convergio-server` CLI safety unit tests | 2 |
| `convergio-i18n` (unit + coverage + doc) | 16 — proves P5 |
| `convergio-api` (unit) | 4 |
| `convergio-mcp` (unit) | 3 |
| `convergio-durability/tests/no_stub_gate.rs` | 17 — proves P4 |
| `convergio-durability/tests/no_secrets_gate.rs` | 4 — proves P2 |
| **Total** | **165** |

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
| Add a new DB table to Layer 1/2/3 | New migration in `crates/<crate>/migrations/` (next free version in your crate's range — see ADR-0003) AND module in `src/store/` |
| Add a new layer / crate that needs tables | Pick the next free hundred for migration versions. Update ADR-0003 index. |
| Add a planner heuristic | `crates/convergio-planner/src/` |
| Document a design decision | `docs/adr/NNNN-short-title.md` (next free number, MADR template). Update `docs/adr/README.md` index. |
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
The **convergio daemon's own MCP surface** (`cvg_*` family) will be
auto-discovered once we ship the MCP server crate (deferred — not in
the MVP scope).

For now the most useful HTTP routes (drive directly via `curl` or
`cvg`):

- Plans: `POST /v1/plans`, `GET /v1/plans/:id`, `GET /v1/plans`
- Tasks: `POST /v1/plans/:plan_id/tasks`, `POST /v1/tasks/:id/transition`
- Evidence: `POST /v1/tasks/:id/evidence`
- Audit: `GET /v1/audit/verify`
- Bus: `POST /v1/plans/:plan_id/messages`, `GET ...?topic=&cursor=`,
  `POST /v1/messages/:id/ack`
- Agents: `POST /v1/agents/spawn`, `POST /v1/agents/:id/heartbeat`
- Layer 4: `POST /v1/solve`, `POST /v1/dispatch`,
  `POST /v1/plans/:id/validate`

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

Two loops run today, one per layer that needs one:

- `Reaper` — `convergio_durability::reaper::spawn`. Default tick 60s,
  default timeout 300s. Releases tasks whose agent stopped heart-beating
  and writes one `task.reaped` audit row per release.
- `Watcher` — `convergio_lifecycle::watcher::spawn`. Default tick 30s.
  Polls `running` rows in `agent_processes` and flips them to `exited`
  when the OS PID is no longer alive (POSIX `kill -0`).

Knobs: `CONVERGIO_REAPER_TICK_SECS`, `CONVERGIO_REAPER_TIMEOUT_SECS`,
`CONVERGIO_WATCHER_TICK_SECS`.

Layer 4 has `convergio_executor::spawn_loop` defined but **not yet
wired** from `main.rs` — for now, the executor is HTTP-triggered via
`POST /v1/dispatch`. Wire it when you're ready (and document the
reason in an ADR).

**Do not document loops you have not actually implemented.** (We had
this exact lie in v2 docs for months — not again.)

## When in doubt

1. Read the relevant `crates/<name>/src/lib.rs` `//!` block.
2. Read the relevant ADR in `docs/adr/`.
3. Read [CONSTITUTION.md](./CONSTITUTION.md).
4. Ask the human. Do not invent.
