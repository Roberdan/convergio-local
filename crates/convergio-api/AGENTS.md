# AGENTS.md — convergio-api

For repo-wide rules see [../../AGENTS.md](../../AGENTS.md).

This crate owns the compact agent action contract used by MCP and future
adapters. It is not the daemon and must not perform IO.

## Invariants

- Keep `convergio.help` and `convergio.act` as the stable agent surface.
- Add actions deliberately; every action becomes prompt/API surface area.
- Keep request/response schemas serializable, versioned, and documented.
- Do not add daemon HTTP calls, database access, or business logic here.
- Dynamic capability actions must be namespaced and schema-validated.
