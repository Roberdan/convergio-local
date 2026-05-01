# AGENTS.md — convergio-lifecycle

For repo-wide rules see [../../AGENTS.md](../../AGENTS.md).

This crate supervises local processes. It is not a sandbox or a full
distributed scheduler.

## Invariants

- Treat spawned commands as untrusted user-level processes.
- Track PID, status, heartbeat, and exit without inventing hidden state.
- Do not claim OS-level isolation unless implemented per platform.
- Runner adapters should be explicit and testable.
- Process events that affect tasks must be visible through daemon/audit
  flows.

## Crate stats

The block below is rewritten by `cvg docs regenerate` (ADR-0015) —
do not edit between the markers.

<!-- BEGIN AUTO:crate_stats -->
**`convergio-lifecycle` stats:** 6 `*.rs` files / 18 public items / 503 lines (under `src/`).

No files within 50 lines of the 300-line cap.
<!-- END AUTO -->
