# Changelog

All notable changes to Convergio will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows Semantic Versioning before 1.0 with explicit
MVP scope notes.

## [Unreleased]

No unreleased changes.

## [0.1.0] - 2026-04-30

### Added

- Initial Convergio Local workspace, with layered Rust crates for DB,
  durability, bus, lifecycle, server, CLI, planner, validator and executor.
- SQLite-backed local daemon, localhost HTTP API, pure HTTP `cvg` CLI and
  one-command local install flow.
- Layer 1 durability: plans, tasks, evidence, gates, reaper and
  hash-chained audit verification.
- Layer 2 bus: persistent local plan-scoped messages with publish, poll and
  ack actions.
- Layer 3 lifecycle: local process spawn, heartbeat and watcher.
- Layer 4 reference flow: planner, executor tick, Thor validator and
  `planner.solve` capability-gated action.
- Server-side gate pipeline, including evidence, wave sequencing, no-debt,
  no-stub, no-secrets and zero-warning gates.
- Guided `cvg demo`, local task/evidence commands, service management,
  setup, doctor diagnostics, MCP logs and `cvg mcp tail`.
- Shared typed agent action contract and stdio MCP bridge with
  `convergio.help` and `convergio.act`.
- CRDT storage foundation for multi-actor row/column state.
- Workspace coordination foundation: resources, leases, patch proposals,
  merge queue arbitration and conflict reporting.
- Durable agent registry, task context packets and plan-scoped bus actions
  for multi-agent coordination through the daemon.
- Local capability registry, Ed25519 signature verification, signed local
  `install-file`, disable and remove safety.
- Constrained local shell runner proof through `spawn_runner`.
- English and Italian Fluent bundles with coverage tests.
- Release artifact workflow, local packaging script, macOS signing and
  notarization documentation.
- Project docs: README, Architecture, Constitution, Roadmap, Security,
  Contributing, Code of Conduct, ADRs and public readiness plan.
- Convergio Community License v1.3 (source-available, aligned with the
  legacy `github.com/Roberdan/convergio` repo).

### Changed

- Repositioned the project as a **single-user, local-first, SQLite-only**
  runtime.
- Removed remote deployment and account-model language from current
  documentation.
- Removed the legacy plan scope field from the plan model, schema, API
  and CLI.
- Added a minimal `convergio start` command parser so `convergio --help`
  works and the documented quickstart is real.
- Removed the unused scaffold-only worktree crate from the workspace.
- Updated README, Architecture, Constitution, Security, Roadmap, ADR
  references and crate READMEs around the focused local MVP.
