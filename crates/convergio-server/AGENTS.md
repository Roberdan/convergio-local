# AGENTS.md — convergio-server

For repo-wide rules see [../../AGENTS.md](../../AGENTS.md).

This crate is the HTTP routing shell around the core layers.

## Invariants

- Routes translate HTTP into layer calls; domain rules live in owning
  crates.
- Axum path params use `:id`, not `{id}`.
- Do not let any route bypass gates, audit, or task ownership checks.
- Keep error responses stable enough for CLI/MCP clients.
- Cross-layer E2E tests belong under this crate's `tests/`.
