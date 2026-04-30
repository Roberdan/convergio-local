# STATUS — where convergio-local is today

**Snapshot:** 2026-04-30, end of the office-hours dogfood marathon.
**Daemon running:** v0.1.2 (T11 + wave-gate-fix live, dogfood-verified).
**Repo legibility score:** 83 / 100 (above floor 70, below target 85; the gap is durability split + 12 near-cap files, both tracked).

This page is the **first thing** an agent or human should read after
`README.md`. It sits next to `ROADMAP.md` (target work) and
`docs/INDEX.md` (file map).

## In one paragraph

Convergio is a local-first daemon that **refuses agent work whose
evidence does not match the claim of done**. v0.1.0 shipped with the
gate pipeline, hash-chained audit, and a basic CLI. The
office-hours dogfood session of 2026-04-30 ran the project as its
own first real user, surfaced ~20 concrete frictions, and closed
the most important: T11 (`done` is set only by Thor) is live, the
demo example exists, the workspace coordination + capability
registry surfaces are in place, and `cvg pr stack` + `cvg pr
queue` keep the merge graph honest. The next milestone is **smart
Thor** (T3.02): the validator runs the project's actual test
pipeline, not just an evidence-shape check.

## What shipped

| Surface | State |
|---|---|
| Gate pipeline (NoDebt, NoStub, ZeroWarnings, NoSecrets, EvidenceGate, WaveSequence) | shipped, all green in production daemon |
| Hash-chained audit (ADR-0002) | shipped, 385+ entries integrity-verified |
| Thor as the only path to `done` (ADR-0011) | shipped (PR #15), live in v0.2.0 |
| `cvg task create` with rich fields (T0) | shipped (PR #22) |
| `cvg pr stack` (T2.03) | shipped (PR #28) — i18n + manifest validation deferred |
| Examples: `examples/claude-skill-quickstart/` (T2.01) | shipped (PR #27) |
| Constitution §13 agent context budget | shipped (PR #17) |
| Constitution §15 worktree discipline | shipped (PR #20) |
| Constitution §16 legibility score | shipped (PR #29) |
| ADR-0012 OODA-aware validation roadmap | shipped (PR #25) |
| `docs/INDEX.md` Tier-1 retrieval | shipped (PR #30) |
| Wave-gate fix (failed = terminal) | shipped (PR #26) |
| Public release v0.2.0 (release-please) | unblocked (Cargo.lock just synced) |

## What is in flight, paused, or queued

| Item | Status | Why paused |
|---|---|---|
| `cvg pr stack` i18n + manifest validation | git stash on `fix/cvg-pr-stack-i18n-and-manifest-validation` | paused for this consolidation wave |
| Plan task T2.04 (auto-close PR → task) | not started | waits for the consolidation to settle |
| Plan task T2.05 (split convergio-durability) | not started | the big architectural piece for v0.3 |
| Plan task T3.02 (smart Thor) | not started | the most important next step strategically |
| Plan task T1.17 (Tier-2 frontmatter + `cvg coherence`) | not started | follows naturally from T1.16 |
| Plan task T4.07 (local RAG over corpus) | future | only when the static index runs out |
| Plan task T4.08 (LLM Wiki, AI-maintained `docs/learnings/`) | future | needs T3.02 + T4.02 first |
| Multi-vendor model routing (T4.04) | future | needs reputation T4.03 first |

## Direction

Convergio v3 is on the trajectory the user named explicitly:

> "Build Convergio while building Convergio. Each round you learn
> first-hand, you find what works, what doesn't, and you improve."

Concrete examples of that loop closing in this session:

- The agent (Claude) registered in `agent_registry`, claimed
  `workspace_lease` on the file it edited, published bus messages
  on the plan, and transitioned tasks through the canonical
  pending → in_progress → submitted lifecycle. Every refusal
  landed in the audit chain. The product was eaten by its first
  real user.
- The legibility audit found durability at 8059 LOC (soft-warn) and
  12 near-cap files: both already tracked as plan tasks before the
  audit existed. The score now keeps the next regression honest.
- The friction log F1-F20 turned every unmet expectation into a
  durable plan task instead of disappearing into chat history.
- ADR-0012 mapped Karpathy's 2026 LLM-Wiki + AutoResearch direction
  onto eight plan tasks (T3.02-3.04 + T4.01-4.05).

## What we are at risk of becoming

The same thing v2 became: **too big to follow**. The legibility
score, `docs/INDEX.md`, the worktree discipline, and the single
dogfood plan are the controls keeping that risk visible. The
consolidation wave that produced this STATUS.md is the reflex
itself — when `cvg status` becomes illegible, stop and fix.

## How to navigate this repo as an agent

1. Read this file (`STATUS.md`).
2. Read [`AGENTS.md`](./AGENTS.md) for the cross-vendor agent rules.
3. Read [`CONSTITUTION.md`](./CONSTITUTION.md) for the
   non-negotiables.
4. Open [`docs/INDEX.md`](./docs/INDEX.md) and pick the doc relevant
   to the task. Do not load the whole repo.
5. If the task is durable, claim it through `cvg task create` on
   plan `8cb75264-8c89-4bf7-b98d-44408b30a8ae` (the office-hours
   plan) so the audit chain captures it.
6. Use `cvg pr stack` before merging anything — it surfaces the
   conflict matrix.

## Plan in one screen

The single durable plan is `8cb75264-8c89-4bf7-b98d-44408b30a8ae`.
Live counts via `cvg status --project convergio-local`. The
distilled queue is in [`ROADMAP.md`](./ROADMAP.md).
