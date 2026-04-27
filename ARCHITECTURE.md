# Architecture

This document describes how Convergio is structured. For the *why* behind
the structure, see [CONSTITUTION.md](./CONSTITUTION.md) and the ADRs in
[`docs/adr/`](./docs/adr/).

## The 4 layers

```
┌─────────────────────────────────────────────────────────────────┐
│  Layer 4 — Reference Implementation                             │
│  planner · thor (validator) · executor · worktree               │
│  CLI: cvg                                                        │
├─────────────────────────────────────────────────────────────────┤
│  Layer 3 — Agent Lifecycle                                      │
│  spawn · supervise · heartbeat                                  │
├─────────────────────────────────────────────────────────────────┤
│  Layer 2 — Agent Communication Bus                              │
│  publish · poll (cursor) · ack · scoped per plan                │
├─────────────────────────────────────────────────────────────────┤
│  Layer 1 — Durability Core                                      │
│  plans · tasks · evidence · agents · audit_log (hash-chained)   │
│  gate pipeline · reaper loop                                    │
├─────────────────────────────────────────────────────────────────┤
│  convergio-db                                                   │
│  sqlx pool + migrations · SQLite (personal) | Postgres (team)   │
└─────────────────────────────────────────────────────────────────┘
```

Lower layers do not depend on higher layers. A LangGraph user who only
wants Layer 1+2+3 deletes the Layer 4 crates and ships.

## Crate map

| Crate | Layer | Public surface | Owns DB tables? |
|-------|-------|----------------|-----------------|
| `convergio-db` | 0 | `Pool`, `Backend` | no |
| `convergio-durability` | 1 | `Durability`, `PlanStore`, `TaskStore`, `EvidenceStore`, `AuditLog`, `gates::*`, `reaper::*` | yes (`plans`, `tasks`, `evidence`, `agents`, `audit_log`) |
| `convergio-bus` | 2 | `Bus::publish`, `Bus::poll`, `Bus::ack` | yes (`agent_messages`) |
| `convergio-lifecycle` | 3 | `Supervisor::spawn`, `heartbeat`, `mark_exited`, `get` | yes (`agent_processes`) |
| `convergio-server` | shell | `router(state)`, `AppState` | no |
| `convergio-cli` | 4 | `cvg` binary | no |
| `convergio-planner` | 4 | `Planner::solve` | no |
| `convergio-thor` | 4 | `Thor::validate` -> `Verdict` | no |
| `convergio-executor` | 4 | `Executor::tick`, `spawn_loop` | no |
| `convergio-worktree` | 4 | (skeleton) | no |

## HTTP surface

All endpoints sit under `/v1`. Errors are
`{ "error": { "code", "message" } }`. Status code mapping:

| Status | Code | When |
|--------|------|------|
| 200 | (success) | normal response |
| 404 | `not_found` | missing plan / task / evidence / message / agent |
| 409 | `gate_refused` | Layer 1 gate refused a task transition |
| 422 | `spawn_failed` | Layer 3 could not `exec` the requested binary |
| 500 | `audit_broken` / `internal` | server-side fault |

| Method | Path | Layer |
|--------|------|-------|
| GET | `/v1/health` | shell |
| POST / GET | `/v1/plans` | 1 |
| GET | `/v1/plans/:id` | 1 |
| POST / GET | `/v1/plans/:plan_id/tasks` | 1 |
| GET | `/v1/tasks/:id` | 1 |
| POST | `/v1/tasks/:id/transition` | 1 (gates) |
| POST | `/v1/tasks/:id/heartbeat` | 1 |
| POST / GET | `/v1/tasks/:id/evidence` | 1 |
| GET | `/v1/audit/verify` | 1 |
| POST / GET | `/v1/plans/:plan_id/messages` | 2 |
| POST | `/v1/messages/:id/ack` | 2 |
| POST | `/v1/agents/spawn` | 3 |
| GET | `/v1/agents/:id` | 3 |
| POST | `/v1/agents/:id/heartbeat` | 3 |
| POST | `/v1/solve` | 4 (planner) |
| POST | `/v1/dispatch` | 4 (executor) |
| POST | `/v1/plans/:id/validate` | 4 (thor) |

## Request lifecycle (Layer 1 transition)

```
client (HTTP POST /v1/tasks/:id/transition)
   │
   ▼
convergio-server (axum router)
   │ extracts path, body, auth context
   ▼
convergio-durability::Durability::transition_task
   │ runs gate pipeline:
   │   PlanStatusGate → EvidenceGate → WaveSequenceGate
   │ on refusal: returns DurabilityError::GateRefused
   ▼ (allowed)
convergio-db::Pool (single transaction)
   │ ① UPDATE tasks SET status = ?, agent_id = ? WHERE id = ?
   │ ② INSERT INTO audit_log (..., prev_hash, hash) VALUES (...)
   ▼
response: 200 { Task } | 409 { error: { code: "gate_refused", ... } }
```

## Modes (single binary)

```rust
match config.db.scheme() {
    "sqlite" => personal_setup(),
    "postgres" => team_setup(),
}
```

- **personal_setup**: SQLite at `~/.convergio/state.db`,
  no auth (localhost bypass), single implicit `org_id = "default"`.
- **team_setup**: Postgres URL, HMAC auth middleware required,
  `org_id` extracted from request header / token. *(Deferred — see
  ROADMAP.)*

The match happens in three places: pool init, migration runner,
auth middleware. Everything else is mode-agnostic.

## Audit hash chain (ADR-0002)

Every row in `audit_log` is:

```text
{
  id: uuid,
  seq: i64,                    // monotonic, 1-based
  prev_hash: hex,              // hash of the previous row (or genesis "0..0")
  payload: canonical_json,     // entity_type, entity_id, transition, agent, ...
  hash: sha256(prev_hash || payload)
}
```

`GET /v1/audit/verify?from=<seq>&to=<seq>` recomputes the chain.
An external cron should call this **with `from=None`** for the strongest
guarantee — ranged verification only catches tampering inside the
range. See `crates/convergio-durability/tests/audit_tamper.rs` for the
proof-of-tamper-detection test suite.

Hash is computed over **canonical** JSON (sorted keys, no whitespace)
to avoid false positives from formatting drift.

## Migration coexistence (ADR-0003)

Each crate owns its tables and ships its own migration files. They
share one `_sqlx_migrations` bookkeeping table by version range:

| Crate | Range |
|-------|-------|
| `convergio-durability` | 1 — 100 |
| `convergio-bus` | 101 — 200 |
| `convergio-lifecycle` | 201 — 300 |
| (next layer) | 301+ |

Every migrator calls `set_ignore_missing(true)` so it does not
complain about rows it didn't write.

## Background loops

Two loops run today (one per layer that needs one):

- **Reaper** — `convergio_durability::reaper::spawn`. Every
  `CONVERGIO_REAPER_TICK_SECS` (default 60s) it scans `tasks` for rows
  in `in_progress` whose `last_heartbeat_at` is older than
  `CONVERGIO_REAPER_TIMEOUT_SECS` (default 300s), releases them back
  to `pending`, clears `agent_id`, and writes one `task.reaped` audit
  row per release.
- **Watcher** — `convergio_lifecycle::watcher::spawn`. Every
  `CONVERGIO_WATCHER_TICK_SECS` (default 30s) it scans `agent_processes`
  rows in `running` and asks the OS via POSIX `kill -0` whether the
  PID is still alive; flips dead ones to `exited`.

Layer 4 has `convergio_executor::spawn_loop` defined but **not yet
wired** from `main.rs` — for now `POST /v1/dispatch` triggers a tick.
Wire it from `main.rs` when you're ready and document the choice in
an ADR.

**Do not document loops you have not actually implemented** — we are
not repeating the v2 "three background loops" lie.

## Where to look for things

- HTTP routes → `crates/convergio-server/src/routes/`
- Schema → `crates/<crate>/migrations/`
- Gate pipeline → `crates/convergio-durability/src/gates/`
- Audit verifier → `crates/convergio-durability/src/audit/`
- Reaper loop → `crates/convergio-durability/src/reaper.rs`
- Bus → `crates/convergio-bus/src/bus.rs`
- Supervisor → `crates/convergio-lifecycle/src/supervisor.rs`
- CLI commands → `crates/convergio-cli/src/commands/`
- E2E tests → `crates/convergio-server/tests/e2e_*.rs`
- ADRs → `docs/adr/`
