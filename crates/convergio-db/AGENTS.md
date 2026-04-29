# AGENTS.md — convergio-db

For repo-wide rules see [../../AGENTS.md](../../AGENTS.md).

This crate owns SQLite connection and migration primitives only.

## Invariants

- SQLite is the only supported database for the local product.
- Keep this crate free of domain logic.
- Do not introduce Postgres/team/tenant abstractions.
- Migrations belong to the crate that owns the table semantics.
- Connection helpers must be safe for concurrent local agents.
