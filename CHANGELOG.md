# Changelog

All notable changes to Convergio will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
