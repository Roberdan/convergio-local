# AGENTS.md — convergio-executor

For repo-wide rules see [../../AGENTS.md](../../AGENTS.md).

This crate is reference dispatch behavior. It should not become the full
orchestrator for every real agent runner.

## Invariants

- Keep deterministic behavior easy to test.
- Real Claude/Copilot/Cursor runner support should be adapter/capability
  work, not hardcoded here.
- Dispatch must respect task claim state and future leases.
- Do not mark tasks done; workers must submit evidence and pass gates.
