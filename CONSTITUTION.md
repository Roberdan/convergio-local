# Convergio Constitution

These rules are non-negotiable. They exist to keep us from drifting into
a generic "agent platform" and to keep the daemon honest.

## 1. Same binary, two modes

There is **one** `convergio` binary. The mode (personal vs team) is a
function of `CONVERGIO_DB`:

- `sqlite://...` (or unset → defaults to `~/.convergio/state.db`) → personal
- `postgres://...` → team

A new mode is **not** a new binary or a new fork. It is a config branch
in three known places (DB pool init, migration selection, auth middleware).

## 2. Cooperate, don't compete

LangGraph, CrewAI, Claude Code skills, AutoGen, Mastra: these are clients,
not competitors. We give them durability, audit and supervision. We do not
ship a DSL, a chain abstraction, or a "Convergio agent framework".

## 3. Reference implementation is part of the product

Layer 4 (`planner`, `thor`, `executor`) ships in the same repo as the
durability layer. It exists so a new user can `convergio start` and see
something useful in 5 minutes. It is not the product — but without it
the product is unsellable.

## 4. Anti-feature creep

These are deferred or cut, period:

- Mesh / multi-host (deferred — until a customer asks)
- Knowledge / catalog / org model (cut — plan + task + evidence is the model)
- Billing (cut — OSS only for now)
- Kernel / MLX (deferred — model agnostic)
- Night agents (deferred — Layer 3 + cron is enough)
- Skills marketplace (cut — never)
- 130+ MCP tools (reduced to ~15 covering layers 1-3 only)

If a feature is not in the 4 layers and not in the roadmap, it does not get
built. Issues are filed in `v3-backlog`.

## 5. Every feature must be tweetable

If explaining a feature requires a diagram, the feature is either not ready
or not the right feature. Ship the explanation first.

## 6. Server-enforced gates only

A task cannot be marked `done` from the client. The daemon verifies evidence
and transitions state. Clients propose, the daemon disposes.

The gate pipeline is fixed:

```
identity → plan_status → evidence → test → pr_commit → wave_sequence → validator
```

Any new gate must be justified, documented in an ADR, and ship with tests.

## 7. Audit log is append-only and hash-chained

Every state transition writes a row to `audit_log` whose `hash` is
`sha256(prev_hash || canonical_json(payload))`. The chain is verifiable
via `GET /v1/audit/verify` from any external process.

Mutating an audit row, or breaking the chain, is a bug, not a feature.

## 8. No SQLite-specific SQL leaks

Schema, migrations and queries must work on both SQLite and Postgres. Where
behavior differs, abstract behind `convergio-db`. CI runs the test suite
against both backends (Postgres added in week 1 of MVP).

## 9. CLI is a pure HTTP client

`cvg` MUST NOT import server crates. It speaks HTTP. A contract test
enforces this.

## 10. Loop must close

Every feature has: input → processing → output → feedback → state update →
visible to the user. If the user can't see the result, it is not done.

## 11. Tests are the spec

If behavior is not under test, it is not guaranteed. Public APIs
(HTTP routes and library `pub fn`) require tests. Bug fixes require a
regression test.
