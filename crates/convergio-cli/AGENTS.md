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

## Crate stats

The block below is rewritten by `cvg docs regenerate` (ADR-0015) —
do not edit between the markers.

<!-- BEGIN AUTO:crate_stats -->
**`convergio-cli` stats:** 40 `*.rs` files / 22 public items / 6046 lines (under `src/`).

Files approaching the 300-line cap:
- `src/commands/session.rs` (298 lines)
- `src/commands/doctor.rs` (259 lines)
- `src/commands/bus.rs` (257 lines)
- `src/commands/capability.rs` (256 lines)
<!-- END AUTO -->
