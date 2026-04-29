# AGENTS.md — convergio-cli

For repo-wide rules see [../../AGENTS.md](../../AGENTS.md).

`cvg` is a human/admin HTTP client. The daemon remains the source of
truth.

## Invariants

- Do not import server crates or write SQLite directly.
- Keep output accessible and useful without color.
- User-facing strings must go through i18n where the command is localized.
- `--output human|json|plain` should be extended consistently.
- CLI convenience must not bypass server-side gates or audit.
