# Changelog

All notable changes to Convergio will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added (sessione 4 — 2026-04-27)

- **Layer 3 OS-watcher** (`convergio_lifecycle::watcher`). Polls every
  `running` row, calls POSIX `kill -0` via `nix::sys::signal::kill`,
  flips dead PIDs to `exited`. Wired from server `main.rs` with
  `CONVERGIO_WATCHER_TICK_SECS` (default 30s). 3 integration tests.
- **Layer 4 — Planner** (`convergio_planner::Planner::solve`). Turns
  a multi-line mission into a plan + one task per non-blank line in
  wave 1. Deterministic, no LLM. 5 tests.
- **Layer 4 — Thor** (`convergio_thor::Thor::validate`). Returns
  `Verdict::Pass` iff every task is `done` with required evidence
  kinds present, else `Verdict::Fail { reasons }`. 4 tests.
- **Layer 4 — Executor** (`convergio_executor::Executor::tick`).
  Picks pending tasks whose wave is ready, spawns agents via Layer 3,
  transitions to `in_progress` with the spawned process id as
  `agent_id`. 4 tests.
- **HTTP routes**: `POST /v1/solve`, `POST /v1/plans/:id/validate`,
  `POST /v1/dispatch`. `ApiError` extended with `From<PlannerError>`
  and `From<ThorError>`.
- **CLI**: `cvg solve`, `cvg dispatch`, `cvg validate`.
- **Quickstart E2E** (`crates/convergio-server/tests/e2e_quickstart.rs`):
  solve a mission → dispatch → force tasks done → validate → assert
  Verdict::Pass. Plus a fail-case test.
- **Cross-layer E2E** (`e2e_full_stack.rs`): drives all 3 lower
  layers in one HTTP-driven workflow.
- **Audit E2E** (`e2e_audit.rs`): 3 dedicated tests (clean verify,
  ranged verify, HTTP detects tampering done via raw SQL).
- README "Project status" + ROADMAP refreshed.

Workspace test count: **68 green** (was 50).

### Added (sessione 3 — 2026-04-27)

- **Audit tamper-detection test suite**
  (`crates/convergio-durability/tests/audit_tamper.rs`, 6 tests).
  Mutates each field of an `audit_log` row via raw SQL and asserts
  `AuditLog::verify` returns `ok=false` with the correct `broken_at`.
  Proves the ADR-0002 security claim end-to-end. These tests are
  load-bearing — red == durability story is broken.
- **Per-gate unit tests** (`crates/convergio-durability/tests/gates.rs`,
  7 tests). PlanStatusGate, EvidenceGate, WaveSequenceGate tested
  individually for refuse + allow + no-op-on-wrong-target.
- **CLI smoke tests** (`crates/convergio-cli/tests/cli_smoke.rs`,
  6 tests, via `assert_cmd` + `predicates`).
- **ADR-0003** "Per-crate migrations on a shared `_sqlx_migrations`
  table" — codifies the version-range convention and the
  `set_ignore_missing(true)` boilerplate.
- ARCHITECTURE.md refreshed: full 18-endpoint table, request
  lifecycle, audit chain, migration coexistence section.
- AGENTS.md refreshed: test-suite layout (46 tests) and the
  migration-version-range rule.

Workspace test count: **46 green** (was 27).

### Added (sessione 2 — 2026-04-27)

- **Layer 1 reaper loop**: `convergio-durability::reaper` releases
  `in_progress` tasks whose `last_heartbeat_at` is older than
  `CONVERGIO_REAPER_TIMEOUT_SECS` (default 300s) and writes one
  `task.reaped` audit row per release. Wired from server `main.rs`.
- **Layer 2 (`convergio-bus`)**: `agent_messages` table, publish +
  cursor-based poll + ack. Per-`(plan_id, topic)` FIFO. Persistent.
  HTTP routes: `POST /v1/plans/:plan_id/messages`, `GET` with cursor,
  `POST /v1/messages/:id/ack`.
- **Layer 3 (`convergio-lifecycle`)**: `agent_processes` table,
  `Supervisor::spawn`/`heartbeat`/`mark_exited`/`get`. HTTP routes:
  `POST /v1/agents/spawn`, `GET /v1/agents/:id`,
  `POST /v1/agents/:id/heartbeat`.
- Migration coexistence: every per-crate migrator calls
  `set_ignore_missing(true)` so durability/bus/lifecycle share the
  `_sqlx_migrations` bookkeeping table without conflict (durability
  owns 1+, bus 101+, lifecycle 201+).
- `ApiError` extended for `BusError` and `LifecycleError` (404 / 422 /
  500 mapping).
- 14 new tests (2 reaper, 5 bus unit, 4 lifecycle unit, 2 e2e bus,
  2 e2e agents) — workspace total 27 tests, all green.

## [0.1.0] - 2026-04-27

### Added

- Initial bootstrap of Convergio v3 reframe (durability layer).
- Cargo workspace with 10 crates matching the 4-layer architecture
  (`db`, `durability`, `bus`, `lifecycle`, `server`, `cli`,
   `planner`, `thor`, `executor`, `worktree`).
- `convergio-db`: sqlx-based pool over SQLite (Postgres deferred behind
  feature flag), migration runner. 5 unit tests.
- `convergio-durability` (Layer 1): plans / tasks / evidence / agents /
  audit_log schema; append-only hash-chained audit log with canonical
  JSON; CRUD via `PlanStore` / `TaskStore` / `EvidenceStore`; gate
  pipeline (`PlanStatusGate`, `EvidenceGate`, `WaveSequenceGate`);
  `Durability` facade that writes one audit row per state-changing
  operation. 6 unit tests.
- `convergio-server`: axum 0.7 routing shell exposing `/v1/*` endpoints
  for plans, tasks, evidence, audit verification, health.
- `convergio-cli` (`cvg`): pure HTTP client with `health`, `plan
  create|list|get`, `audit verify`. Zero internal imports — contract
  test territory ready.
- Skeleton crates for Layer 2 (`bus`), Layer 3 (`lifecycle`) and the
  Layer 4 reference implementation (`planner`, `thor`, `executor`,
  `worktree`).
- End-to-end test: boots the router in-process, runs the full plan →
  task → evidence → submitted lifecycle, verifies that the gate
  pipeline refuses with HTTP 409 on missing evidence, and confirms the
  audit chain verifies clean.
- Apache 2.0 license.
- Foundation docs (agent-first): AGENTS.md as cross-vendor single source
  of truth with symlinked CLAUDE.md and `.github/copilot-instructions.md`,
  README, ARCHITECTURE, CONTRIBUTING, CONSTITUTION (11 non-negotiable
  rules), ROADMAP, SECURITY, CODE_OF_CONDUCT.
- Tooling: `.cursor/rules`, `.claude/settings.json`, `.mcp.json`,
  `.cursorignore`, `lefthook.yml`, `commitlint.config.js`,
  `rust-toolchain.toml`, `.cargo/config.toml`, GitHub Actions CI,
  Dependabot, PR template enforcing 5-section body.
- ADR-001 (four-layer architecture), ADR-002 (audit hash chain), MADR
  template.
- Original v3 specification archived under `docs/spec/`.
