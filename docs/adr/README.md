# Architecture Decision Records

We document load-bearing decisions in [MADR](https://adr.github.io/madr/)
format. Numbering is monotonic — never reuse a number.

## Workflow

1. Copy `0000-template.md` to `NNNN-short-title.md` (next free number).
2. Fill in Context, Drivers, Options, Decision.
3. Status starts at `proposed`. PR review flips it to `accepted` or `rejected`.
4. If a later decision overrides this one, set status to `superseded by NNNN`.

## Index

| # | Title | Status |
|---|-------|--------|
| [0001](0001-four-layer-architecture.md) | Four-layer architecture | accepted |
| [0002](0002-audit-hash-chain.md) | Hash-chain the audit log | accepted |
| [0003](0003-migration-coexistence.md) | Per-crate migrations with version-range convention | accepted |
| [0004](0004-three-sacred-principles.md) | Three sacred principles (zero tolerance, security, accessibility) | accepted |
| [0005](0005-internationalization-first.md) | Internationalization first (P5) — Italian + English day one | accepted |
| [0006](0006-crdt-storage.md) | Model state with row and column CRDT metadata from day zero | proposed |
| [0007](0007-workspace-coordination.md) | Coordinate multi-agent workspace changes with leases and patch proposals | proposed |
| [0008](0008-downloadable-capabilities.md) | Install new behavior as signed isolated capabilities | proposed |
| [0009](0009-agent-client-protocol-adapter.md) | Treat Agent Client Protocol as a future northbound editor adapter | proposed |
| [0010](0010-retire-convergio-worktree-crate.md) | Retire the convergio-worktree crate | accepted |
| [0011](0011-thor-only-done.md) | Done is set only by Thor (the validator) | accepted |
| [0012](0012-ooda-aware-validation.md) | OODA-aware validation: outcome reliability over output reliability | accepted |
| [0013](0013-split-durability-into-three-crates.md) | Split convergio-durability along three seams (audit / state / coordination) | proposed |
