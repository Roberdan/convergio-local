# Changelog

All notable changes to Convergio will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows Semantic Versioning before 1.0 with explicit
MVP scope notes.

## [Unreleased]

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

### Current local runtime

- Layer 1 durability: plans, tasks, evidence, gates, reaper and
  hash-chained audit verification.
- Layer 2 bus: persistent local messages with poll/ack.
- Layer 3 lifecycle: local process spawn, heartbeat and watcher.
- Layer 4 reference flow: planner, executor tick, Thor validator and
  `cvg` CLI.
- Internationalization: English and Italian Fluent bundles with coverage
  tests.

## [0.1.0] - 2026-04-27

### Added

- Initial Convergio v3 workspace.
- Layered Rust crates for DB, durability, bus, lifecycle, server, CLI,
  planner, validator and executor.
- SQLite-backed local state.
- Hash-chained audit log with tamper-detection tests.
- Server-side gate pipeline, including no-debt, no-stub and
  zero-warning gates.
- Local HTTP API and pure HTTP CLI.
- E2E tests for the local daemon workflow.
- Project docs: README, Architecture, Constitution, Roadmap, Security,
  Contributing, Code of Conduct and ADRs.
- Convergio Community License v1.3 (source-available, aligned with the
  legacy `github.com/Roberdan/convergio` repo).
