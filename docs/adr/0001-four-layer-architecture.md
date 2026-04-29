# 0001. Adopt a four-layer architecture (durability, bus, lifecycle, reference)

- Status: accepted
- Date: 2026-04-26
- Deciders: Roberto, Office-Hours session 2
- Tags: foundation

## Context and Problem Statement

Convergio v2 grew to 38 crates spread across multiple repos. Many of them
solved problems nobody had asked for (mesh multi-host, knowledge graph,
billing) while the core durability properties (state, audit, supervision)
were never sold as a unit. The repo structure made it impossible for new
contributors — human or agent — to know what was load-bearing.

We need a structure where:

1. The core value (durability + audit) is unmistakably the product.
2. Everything else either builds on top, or is cut.
3. Clients in any framework (LangGraph, CrewAI, Claude Code, plain Python)
   can adopt only what they need.

## Decision Drivers

- Local-first SQLite runtime — one daemon, one user, one database file.
- Cooperate with existing agent frameworks, don't compete.
- Reference implementation must ship in the same repo so adoption is
  5-minute, not "go build the client too".
- Anti-feature creep — the structure itself must make it hard to add
  non-load-bearing features.

## Considered Options

1. **Plugin / extension trait architecture** (v2 today) — every feature is
   an `Extension`. 38 crates, 70+ ADRs. Hard to know what's core.
2. **Monolithic single crate** — everything in one `convergio` crate.
   Easy to grok, hard to compose.
3. **Four explicit layers**: Durability Core → Comm Bus → Lifecycle →
   Reference Implementation. Lower layers don't depend on higher ones.
4. **Three layers** (collapse Lifecycle into Durability) — simpler but
   muddles "data" with "process".

## Decision Outcome

Chosen option: **3 — Four explicit layers**. The layer boundary is the
unit of independence: a user can adopt Layer 1 alone, Layer 1+2, or all
four.

### Positive consequences

- The product is "Layer 1+2+3" — Layer 4 is a quickstart, not the value.
- Each layer is small enough to grok in one sitting.
- Crate dependencies are a DAG, not a hairball.
- Anti-feature creep is structural: if a feature doesn't fit a layer,
  it doesn't ship.

### Negative consequences

- Some refactoring of v2 code that conflated layers.
- Tempting to "cheat" the boundaries — Layer 1 could call out to Layer 4
  and we'd "save code". We won't. The layer boundary is enforced by
  crate dependency graph (`cargo metadata` in CI).

## Links

- Spec: [docs/spec/v3-durability-layer.md](../spec/v3-durability-layer.md)
- Related: ADR-0002 (audit hash chain)
