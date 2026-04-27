# Changelog

All notable changes to Convergio will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-04-26

### Added

- Initial bootstrap of Convergio v3 reframe (durability layer).
- Cargo workspace with 10 crates matching the 4-layer architecture
  (`db`, `durability`, `bus`, `lifecycle`, `server`, `cli`,
   `planner`, `thor`, `executor`, `worktree`).
- `convergio-db`: sqlx-based abstraction over SQLite (and prepared
  for Postgres via feature flag), pool, migration runner.
- `convergio-durability` (Layer 1): plans/tasks/evidence/agents/audit_log
  schema, append-only hash-chained audit log with external verifier,
  CRUD + state transition with gate pipeline skeleton.
- `convergio-server`: axum 0.7 routing shell exposing `/v1/*` endpoints
  for plans, tasks, evidence, audit verification, health.
- `convergio-cli` (`cvg`): clap-derive HTTP client for `start`, `health`,
  `plan create|list|tree`, `audit verify`.
- E2E test that boots an in-process server, creates a plan + task,
  attaches evidence, transitions state, verifies the audit chain.
- Apache 2.0 license.
- Foundation docs: README, ARCHITECTURE, CONTRIBUTING, CONSTITUTION,
  ROADMAP, ADR-001 (layered architecture), ADR-002 (audit hash chain).
- Original v3 specification archived under `docs/spec/`.
