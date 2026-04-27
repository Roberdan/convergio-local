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
