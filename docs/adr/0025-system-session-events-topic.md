---
id: 0025
status: accepted
date: 2026-05-01
topics: [bus, layer-2, schema, multi-agent, session-events]
related_adrs: [0001, 0009, 0011, 0018]
touches_crates: [convergio-bus, convergio-server, convergio-mcp, convergio-api]
last_validated: 2026-05-01
---

# 0025. The agent message bus accepts a `system.*` topic family with `plan_id IS NULL`

- Status: accepted
- Date: 2026-05-01
- Deciders: Roberdan
- Tags: bus, schema, session-events

## Context and Problem Statement

ADR-0001 defines Layer 2 (the agent message bus) as
*plan-scoped*: every message belongs to a `plan_id`. The schema
in `crates/convergio-bus/migrations/0101_bus_init.sql` enforces
this with `plan_id TEXT NOT NULL`. The model docs in
`crates/convergio-bus/src/lib.rs` say plainly:

> *Messages are scoped per plan_id. System-wide messaging is
> out of scope for v1.*

That choice was correct for v0.1.x, when every agent operation
already lived inside a plan. Wave 0b (PRD-001 Claude Code
adapter) breaks that assumption.

A Claude Code session at startup is **not yet attached to a
plan**: the human has not picked one, the agent has not claimed
a task. PRD-001 needs the session to publish presence signals
(`agent.attached`, `agent.heartbeat`, `agent.idle`,
`agent.detached`) *immediately on session start*, before any
plan context exists. The same applies to peer-discovery messages
("hello, I'm here, anyone else around?") and to administrative
events (lease-claimed, lease-released) that may belong to no
single plan.

Concrete dogfood evidence captured in this very session
(2026-05-01): two Claude Code sessions ran on the same repo for
hours with no awareness of each other, because neither had a
shared plan to publish on. When coordination finally happened
manually, each agent picked a *different* plan as a substitute
(plan v0.1.x topic `system.coordination` for one, plan v0.2 topic
`coordination/agents` for the other). The convention diverged.
The audit chain has the messages but they are scattered across
plan rooms by coincidence, not design.

The fix is small but structural: introduce a single
**system-topic family** with `plan_id IS NULL` semantics, and
codify the convention so future agents publish session-level
events in *one* place.

## Decision Drivers

- **PRD-001 cannot ship without it.** The skill `/cvg-attach`
  needs an agent.attached publish channel that does not depend
  on the human having picked a plan first. There is no other
  place the message can go.
- **Convention beats coincidence.** Without a named topic family,
  every agent invents its own, the audit chain becomes archaeology
  instead of forensics. A single system topic family ends the
  drift.
- **Modulor compatibility (CONSTITUTION § 17).** A bus message
  belongs to the `(task, evidence, gate, audit_row)` Modulor
  the moment it produces an audit row. Removing the
  plan_id-NOT-NULL constraint for a *narrow* class of messages
  preserves the rule everywhere else and only opens a hole for
  the system family.
- **Reversibility.** The change is a NULLability relaxation +
  CHECK constraint. If we are wrong, the migration backout is
  cheap (remove CHECK, set NULL rows back, re-impose NOT NULL).

## Considered Options

### Option A — Reject; force every agent to live inside a plan

The Claude Code skill must `/cvg-attach <plan_id>` and refuse
attach if no plan is provided. Costs: every Claude Code session
that wants Convergio coordination has to first pick a plan,
even when the human has no plan yet (e.g. exploratory
debugging). Friction kills adoption; the dogfood already
demonstrated it.

### Option B — A separate "system bus" table

Create a new `system_events` table parallel to `agent_messages`.
Costs: doubles the bus surface, splits the audit, and forces
clients to learn two APIs. ADR-0001 four-layer architecture
discourages this kind of fork.

### Option C — Allow `plan_id IS NULL` on the existing table, gated by a `system.*` topic prefix CHECK constraint (chosen)

Keep one bus table, one API surface, one audit chain. Open the
NULL gate only for messages whose `topic` starts with `system.`.
Define a closed initial set of allowed system message kinds
(extensible by ADR). Retention semantics (24h ring buffer) and
audit semantics (every message audited) are inherited from the
existing bus.

## Decision Outcome

Chosen option: **Option C**, because it is the smallest change
that unblocks PRD-001 without forking the bus and without
forcing pseudo-plan ceremony on Claude Code sessions.

### Schema (Wave 0b migration `0103_system_topics.sql`)

```sql
-- Bus migration 0103: relax plan_id for system.* topics.
-- See ADR-0023.

-- 1. Drop the NOT NULL on plan_id by recreating the table.
--    SQLite does not support ALTER COLUMN DROP NOT NULL directly;
--    the standard idiom is table-rebuild. The migration preserves
--    every row, every index, and the seq sequence.

CREATE TABLE agent_messages_new (
    id            TEXT PRIMARY KEY,
    seq           INTEGER NOT NULL,
    plan_id       TEXT,                              -- now nullable
    topic         TEXT NOT NULL,
    sender        TEXT,
    payload       TEXT NOT NULL,
    consumed_at   TEXT,
    consumed_by   TEXT,
    created_at    TEXT NOT NULL,
    -- Constraint: plan_id may be NULL only for system-topic messages.
    CHECK (
        plan_id IS NOT NULL
        OR topic LIKE 'system.%'
    )
);

INSERT INTO agent_messages_new
SELECT id, seq, plan_id, topic, sender, payload,
       consumed_at, consumed_by, created_at
FROM agent_messages;

DROP TABLE agent_messages;
ALTER TABLE agent_messages_new RENAME TO agent_messages;

CREATE UNIQUE INDEX idx_agent_messages_seq
    ON agent_messages (seq);
CREATE INDEX idx_agent_messages_plan_topic
    ON agent_messages (plan_id, topic, consumed_at);
CREATE INDEX idx_agent_messages_system_topic
    ON agent_messages (topic, created_at)
    WHERE plan_id IS NULL;
```

### Initial allowed `system.*` message kinds

Closed set, extensible by small ADR. Each kind has a documented
payload shape (full schemas live in
`crates/convergio-bus/src/bus_system.rs` once Wave 0b ships).

| Topic | Kind | Purpose |
|---|---|---|
| `system.session-events` | `agent.attached` | session start presence broadcast |
| `system.session-events` | `agent.heartbeat` | periodic liveness |
| `system.session-events` | `agent.idle` | agent waiting on human input |
| `system.session-events` | `agent.detached` | clean session exit |
| `system.session-events` | `agent.detached_with_known_gaps` | exit via `pre-stop --force` (PRD-001 Artefact 4) |
| `system.session-events` | `agent.lease-claimed` | workspace lease acquired (across plans) |
| `system.session-events` | `agent.lease-released` | workspace lease released |
| `system.coordination` | `presence-announce` | discovery message to peer agents |
| `system.coordination` | `handshake` | declared scope ("about to touch X, Y") |
| `system.coordination` | `status-update` | progress update |
| `system.coordination` | `ack-and-status-update` | reply to a handshake or status-update |

Topics outside this set with the `system.` prefix are *valid at
the schema level* (the CHECK accepts them) but **agent-side
unsupported** until added by ADR. The bus accepts; the client
SDK ignores. This keeps the schema permissive while the
convention stays curated.

### Retention

System-topic messages share the same retention as plan-scoped
messages: 24h ring buffer of unconsumed entries with archival to
the audit chain on consume / on retention sweep. Roadmap item:
configurable retention per topic family (post-Wave-0b).

### Audit semantics

Every system-topic message produces an audit row exactly as
plan-scoped messages do. The audit row carries
`plan_id: null` and a topic field; the hash chain is unaffected.

### `Bus::poll(plan_id, ...)` API surface

Wave 0b extends the existing API:

- `Bus::poll(Some(plan_id), topic, ...)` — current behaviour,
  unchanged.
- `Bus::poll(None, topic, ...)` — **new**, polls system-topic
  messages. Refuses if `topic` does not start with `system.`.

The MCP `poll_messages` action accepts `plan_id` as nullable
*only when* `topic` starts with `system.`; refuses otherwise
with `invalid_request: plan_id required for non-system topic`.
This is enforced server-side in `convergio-server`, not just
client-side, so any future client gets the same protection.

### What this decision does not do

- It does not turn the bus into a system-event log. The vast
  majority of bus traffic stays plan-scoped. System topics are
  the *exception channel* for events that genuinely have no
  plan home.
- It does not add multi-tenant or remote semantics. Local-first
  remains the architectural commitment (CONSTITUTION).
- It does not deprecate `plan_id`-required topics. They remain
  the default and the right choice for any agent operation
  that *does* belong to a plan.
- It does not implement the Wave 0b notification surfacing or
  pre-stop check. Those live in PRD-001 and use this ADR's
  primitives.

## Consequences

### Positive

- PRD-001 unblocked. The skill `/cvg-attach` can publish
  presence signals on session start without ceremony.
- Convention codified. Future agents publish session-level
  events on `system.session-events` and coordination on
  `system.coordination`. The audit chain becomes
  forensics-grade for cross-session analysis.
- Two of the nine gaps from the dogfood review (2026-05-01) are
  now structurally addressable: gap 1 (no proactive bus
  polling) and gap 9 (bus requires plan_id even when no plan
  exists). The remaining seven gaps stay in PRD-001 scope.
- Backward compatibility intact. Existing plan-scoped bus
  traffic is unaffected.

### Negative

- Schema rebuild migration. SQLite cannot relax NOT NULL
  in-place; the migration recreates the table. For a
  single-user local DB this is fine; the migration is small and
  fast. We accept the operational cost.
- New CHECK constraint surface. A future migration mistake that
  forgets the CHECK could quietly let `plan_id IS NULL` in for
  non-system topics. Mitigation: a unit test in
  `convergio-bus` asserts the constraint exists and rejects a
  rogue insert.
- Topic-prefix governance. Adding new system topics or kinds
  requires an ADR (small ADR is fine). This is intentional
  friction to prevent system-topic sprawl, but it is friction.

### Neutral

- The MCP shape changes are minor and additive. `publish_message`
  already accepts `sender`/`payload`/`topic` as documented;
  `plan_id` becoming nullable for `system.*` topics is the only
  delta.

## Validation

This ADR is validated when:

1. After Wave 0b migration `0103_system_topics.sql` ships,
   `INSERT INTO agent_messages (..., plan_id, topic, ...)
   VALUES (..., NULL, 'system.session-events', ...)`
   succeeds.
2. The same insert with `topic = 'plan.task.something'`
   (non-system prefix) **fails** the CHECK constraint.
3. `Bus::poll(None, "system.session-events", ...)` returns
   matching rows.
4. `Bus::poll(None, "task.something", ...)` returns
   `invalid_request`.
5. `cvg audit verify` returns `ok=true` after a session that
   publishes system-topic messages and then exits via `cvg
   session pre-stop`.
6. The dogfood demonstration in `examples/skills/cvg-attach/`
   shows two Claude Code sessions exchanging
   `system.coordination` `presence-announce` + `handshake` and
   their audit rows are queryable via
   `cvg audit list --topic system.*`.
