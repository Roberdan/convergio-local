---
id: 0014
status: proposed
date: 2026-05-01
topics: [layer-1, retrieval, graph, context]
related_adrs: [0001, 0002, 0007, 0011, 0012, 0013]
touches_crates: [convergio-graph, convergio-cli, convergio-server, convergio-durability]
last_validated: 2026-05-01
---

# 0014. Code-graph layer for Tier-3 context retrieval

- Status: proposed
- Date: 2026-05-01
- Deciders: Roberto, claude-code-roberdan
- Tags: layer-1, retrieval, graph, context

## Context and Problem Statement

Convergio's whole product premise is that **the agent's work fails the
gate when its evidence does not match the claim**. The premise breaks
when the agent does not have enough context to make a true claim in
the first place. Today the agent's context loop is two-tier:

- **Tier 1** — `docs/INDEX.md` (auto-generated file map). Coarse but
  always current.
- **Tier 2** — `cvg coherence check` (ADR frontmatter cross-validated
  against `workspace.members` and the README index). Checks
  *declared* relationships, never actual ones.

Two problems remain that the existing tiers cannot solve:

1. **Doc/code drift.** An ADR says `touches_crates: [convergio-audit]`
   but the diff implementing it actually edits `convergio-mcp` too.
   Today nothing surfaces this. Tomorrow it surfaces only after a
   reviewer notices.
2. **Context-pack delegation.** When we hand a task to a subagent
   (Sonnet, Copilot, Codex), we either dump the whole repo (token
   blowout, garbage output) or hand-curate a slice (fragile, slow,
   does not scale). We need an **automatic context-pack scoped to a
   task**, computed from the actual code graph.

Without a third tier, every delegation is russian-roulette and every
ADR is a promise the gate can not enforce.

## Decision Drivers

- **Rust-only, no scripts.** The user has called this out explicitly.
  A new shell script per concern is a smell; a new `cvg` subcommand
  backed by a Rust crate is the right shape.
- **Local-first, single SQLite.** State persists where every other
  layer's state lives.
- **Sub-second on a warm cache.** A Tier-3 query that takes longer
  than the agent's typing speed is dead on arrival.
- **Sees private items.** The use cases are internal refactors and
  cluster splits, not API surface analysis. Anything that hides
  `pub(crate)` and below is the wrong primitive for v0.
- **Closes the loop with existing tiers.** Tier-1 (file map) feeds
  the parser, Tier-2 (frontmatter) is the *declared* truth, Tier-3
  (graph) is the *actual* truth. Drift detection is the diff.

## Considered Options

1. **`syn` parse-only + `cargo_metadata`.** Walk every `*.rs` in the
   workspace via `syn`, extract module tree, `use` graph, item
   declarations (struct, fn, mod, trait), and call sites. Combine
   with `cargo metadata --format-version 1` for crate-level
   dependency edges.
2. **`rustdoc --output-format json`.** Run `cargo doc` with the
   stable JSON formatter; parse the resulting tree. Get fully
   type-resolved API graph.
3. **`rust-analyzer` as a library** (`ra_ap_*` crates). Full LSP
   semantic analysis available in-process.
4. **`tree-sitter` + `tree-sitter-rust`.** Language-agnostic parser
   producing CSTs.

## Decision Outcome

**Chosen option: 1 (syn + cargo_metadata) for v0.** Layer (2) on top
in v1 when we need type-resolved transitive analysis.

### Why not the others

- **rustdoc JSON** sees only `pub` items. Internal refactors (the
  primary v0 use case) are invisible. Also requires a full
  `cargo doc` run per query — orders of magnitude slower than parsing
  a single file.
- **rust-analyzer-as-lib** is the most powerful option and the most
  expensive. The `ra_ap_*` crates are unstable, the dependency
  closure is large (~hundreds of crates), and we would inherit any
  RA breakage. Wrong default for a Layer-1 utility crate.
- **tree-sitter** is great for polyglot tooling but Convergio is
  intentionally Rust-only. `syn` understands edition rules, macro
  invocations, and attribute syntax that `tree-sitter-rust` skips.

### What syn-first buys us

- Parse one file in milliseconds.
- See every item including `pub(crate)`, `pub(super)`, private.
- Walk `use` paths verbatim (good enough for "what does this file
  reference"; we explicitly do *not* try to do name resolution).
- Zero runtime dependencies — `syn` is a build-time crate already in
  most workspaces' transitive closure.

### What syn-first does NOT do (intentional v0 scope)

- No name resolution. `Foo::bar` in module `m` is recorded as the
  unresolved path `Foo::bar`, not as the canonical `crate::a::Foo::bar`.
- No type resolution. Function return types are stored as their
  written form, not their definitive type.
- No macro expansion. `#[derive]` and procedural macros are treated
  as opaque attributes — their generated code is not parsed.

These limitations are documented in the API; users wanting deeper
analysis run `cvg graph build --rustdoc` (v1) for a slower, deeper
pass.

## Topology

```
crates/
├── convergio-graph/          ← new Layer-1 sibling
│   ├── Cargo.toml
│   ├── AGENTS.md
│   ├── CLAUDE.md → AGENTS.md
│   ├── migrations/
│   │   └── 0600_graph_nodes_edges.sql
│   └── src/
│       ├── lib.rs
│       ├── parse.rs          ← syn walker
│       ├── meta.rs           ← cargo_metadata wrapper
│       ├── doc_link.rs       ← ADR/markdown ↔ symbol edges
│       ├── store.rs          ← SQLite persistence + queries
│       ├── refresh.rs        ← lazy-on-read mtime check + opt-in loop
│       └── model.rs          ← Node, Edge, ContextPack, DriftReport
└── convergio-cli/
    └── src/commands/graph.rs ← cvg graph build|for-task|cluster|drift
```

`convergio-server` mounts a router for the HTTP surface. `convergio-durability` is **not** modified — graph storage lives in its own SQLite tables under migration range 600-699 (see ADR-0003).

## Schema (migration 0600)

```sql
CREATE TABLE graph_nodes (
  id           TEXT PRIMARY KEY,    -- stable hash of (kind, path, name)
  kind         TEXT NOT NULL,       -- crate | module | item | adr | doc
  name         TEXT NOT NULL,
  file_path    TEXT,                -- NULL for adr/doc nodes
  crate_name   TEXT NOT NULL,       -- 'docs' for non-code nodes
  span_start   INTEGER,             -- byte offset, NULL for non-code
  span_end     INTEGER,
  last_parsed  TEXT NOT NULL,       -- ISO-8601 UTC
  source_mtime TEXT NOT NULL        -- file mtime at parse time
);

CREATE TABLE graph_edges (
  src      TEXT NOT NULL REFERENCES graph_nodes(id),
  dst      TEXT NOT NULL REFERENCES graph_nodes(id),
  kind     TEXT NOT NULL,           -- uses | declares | re_exports | claims | mentions
  weight   INTEGER NOT NULL DEFAULT 1,
  PRIMARY KEY (src, dst, kind)
);

CREATE INDEX idx_graph_edges_dst ON graph_edges(dst, kind);
CREATE INDEX idx_graph_nodes_file ON graph_nodes(file_path);
```

## API surface

- `cvg graph build [--force]` — full or incremental rebuild.
- `cvg graph for-task <task_id> [--max-tokens N]` — emits a JSON
  context-pack: relevant files, top-K symbols, related ADRs,
  recent audit events on those files. Default cap 8000 tokens.
- `cvg graph cluster <crate> [--target-loc 4000]` — community
  detection on the symbol graph; suggests split seams.
- `cvg graph drift [--since <git-ref>]` — diff `frontmatter.touches_crates`
  vs the actual crates touched in the diff.

HTTP equivalents under `/v1/graph/*` for daemon-driven clients.

## Auto-update strategy

Three layers, opt-in stack:

1. **Lazy on read** (always on). Each `graph_nodes` row stores the
   source file mtime at parse time. On any `cvg graph for-task`
   call, the parser re-runs against any file whose mtime is newer
   than its row.
2. **Opt-in daemon refresh loop.** New `convergio_graph::refresh::loop`
   ticks every `CONVERGIO_GRAPH_REFRESH_SECS` (default off). For
   maintainers who want instant feedback during heavy editing.
3. **Lefthook post-commit nudge.** Hook fires
   `curl -fsS -X POST localhost:8420/v1/graph/refresh` after every
   commit. No-op if the daemon is down.

## Drift semantics — advisory v0, gate v1

v0 ships `cvg graph drift` as an advisory CI step (same shape as
`cvg coherence check`). It computes:

```
declared = ⋃ ADR.touches_crates  for ADRs referenced in the PR body or commits
actual   = { crate(file) : file ∈ git diff name-only }
drift    = actual ∖ declared    (crates touched but not declared)
ghosts   = declared ∖ actual    (crates declared but not touched)
```

v1 (after we have data on false-positive rates) promotes the check
to a server-side gate that refuses `submitted` when drift > 0.

## Interaction with existing primitives

- **`cvg coherence check` (T1.17)**: stays. It is the *declarative*
  audit (frontmatter parses correctly, references resolve to known
  ADRs/crates). The graph is the *empirical* audit (the code agrees
  with the declaration).
- **`cvg session resume` (T1.23)**: gains an optional `--task-id`
  flag that prepends the context-pack from `cvg graph for-task` to
  the brief.
- **CONSTITUTION § 16 legibility**: cluster detection feeds the
  near-cap heuristic — a crate at 4500 LOC with 3 tight communities
  is more legible than one at 4500 LOC with one giant blob.

## Migration plan

Three PRs:

- **PR 14.1** — `convergio-graph` crate scaffold + `parse.rs` + `meta.rs`
  + `store.rs` + migration 0600 + `cvg graph build`. No CLI features
  beyond `build` and `for-task`. Acceptance: `cvg graph build` on this
  workspace completes < 5s and reports stable counts.
- **PR 14.2** — `cvg graph cluster` + `cvg graph drift` + lefthook
  post-commit nudge + advisory CI step. Acceptance: drift on a
  synthetic ADR claim mismatch is detected.
- **PR 14.3** — `cvg session resume --task-id` integration + opt-in
  refresh loop. Acceptance: a subagent receiving the context-pack
  produces a smaller diff than one receiving the whole repo for the
  same task.

## Pros and Cons of the Options

### Option 1 (chosen)

- 👍 Fast, sees private items, zero new runtime deps.
- 👍 Aligns with Rust-only constraint.
- 👎 No type resolution; some queries must do conservative path matching.

### Option 2 (rustdoc JSON)

- 👍 Type-resolved, official.
- 👎 Public-only, slow, requires a full compile per refresh.

### Option 3 (rust-analyzer as lib)

- 👍 Most accurate.
- 👎 Unstable internal API, large dep closure, wrong default for Layer 1.

### Option 4 (tree-sitter)

- 👍 Language-agnostic.
- 👎 Less accurate on Rust edition / macro / attribute syntax.

## Links

- Plan tasks (added 2026-05-01): "Code-graph engine: convergio-graph
  crate (syn-based, Tier-3 retrieval)" and this ADR's companion task,
  both wave 3 of plan `8cb75264-8c89-4bf7-b98d-44408b30a8ae`.
- Related ADRs: 0001 (four-layer architecture), 0002 (audit chain),
  0007 (workspace coordination), 0011 (Thor-only-done), 0012 (OODA
  validation), 0013 (durability split).
- Karpathy 2026 LLM-Wiki note (in ADR-0012) — same direction:
  context-pack > context-dump.
