# 0010. Retire the convergio-worktree crate

- Status: accepted
- Date: 2026-04-30
- Deciders: Roberdan, office-hours dogfood session
- Tags: layer-4, repo-hygiene

## Context and Problem Statement

`crates/convergio-worktree/` was created during early v3 bootstrap as a
placeholder for a Layer 4 git-worktree integration. It never reached an
implementation: no `Cargo.toml`, no source files, no tests, never wired
into `[workspace.members]` in production.

The v0.1.0 release explicitly removed it from `Cargo.toml` (see
CHANGELOG entry *"Removed the unused scaffold-only worktree crate
from the workspace"*). The on-disk `crates/convergio-worktree/`
directory survived that change as an empty husk: one empty `src/`
subdirectory, no other files.

Constitution P4 forbids scaffolding-only artifacts; Constitution §11
requires every crate under `crates/` to ship `AGENTS.md` and
`CLAUDE.md`. The current husk fails both tests. It is a
documentation drift waiting to confuse the next contributor.

## Decision Drivers

- P4 (no scaffolding only) applies to repository structure, not just
  agent evidence — empty crate skeletons are scaffolding by another
  name.
- §11 (mandatory crate AGENTS.md/CLAUDE.md) cannot be satisfied for
  a crate with no source and no purpose.
- Future Layer 4 git-integration work, if needed, has better homes:
  `convergio-executor` (where the dispatch loop lives), or a future
  capability package per ADR-0008.
- ADR-0007 (workspace coordination) and ADR-0008 (downloadable
  capabilities) cover the parallel-agents-on-one-worktree concern
  this crate would have addressed.

## Considered Options

1. **Revive** the crate: add `Cargo.toml`, `lib.rs`, `AGENTS.md`,
   `CLAUDE.md`, register in `[workspace.members]`, give it real
   purpose tied to a Layer 4 use case.
2. **Delete** the directory entirely. Document the retirement here.
3. **Leave as-is**. Continue to ship the husk.

## Decision Outcome

Chosen option: **Option 2 — delete**, because:

- Reviving requires a use case the project does not have today.
  ADR-0007 and ADR-0008 cover the same problem space without a
  dedicated crate.
- §11 makes "leave as-is" non-compliant on a permanent basis.
- The CHANGELOG already announced the retirement to users; the
  filesystem should match the changelog.

### Positive consequences

- `crates/` directory listing is now an honest enumeration of real
  crates.
- Future contributors do not have to wonder what the empty directory
  was for.
- Repository self-consistency: CHANGELOG, `Cargo.toml`, and on-disk
  layout agree.

### Negative consequences

- If a future capability does want a `convergio-worktree` name,
  numbering convention loses no information (no ADR ever pointed at
  this crate name as load-bearing). A future crate can reuse the
  name freely.

## Links

- CHANGELOG entry "Removed the unused scaffold-only worktree crate
  from the workspace" under [0.1.0].
- Related: [ADR-0007](0007-workspace-coordination.md),
  [ADR-0008](0008-downloadable-capabilities.md).
- Office-hours plan task T6 on plan
  `8cb75264-8c89-4bf7-b98d-44408b30a8ae`.
