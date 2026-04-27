# Architecture

This document describes how Convergio is structured. For the *why* behind
the structure, see [CONSTITUTION.md](./CONSTITUTION.md) and the ADRs in
`docs/adr/`.

## The 4 layers

```
┌─────────────────────────────────────────────────────────────────┐
│  Layer 4 — Reference Implementation                             │
│  planner · thor (validator) · executor · worktree               │
│  CLI: cvg                                                        │
├─────────────────────────────────────────────────────────────────┤
│  Layer 3 — Agent Lifecycle                                      │
│  spawn · supervise · heartbeat · reap                           │
├─────────────────────────────────────────────────────────────────┤
│  Layer 2 — Agent Communication Bus                              │
│  topic + direct messaging · ack · scoped per plan               │
├─────────────────────────────────────────────────────────────────┤
│  Layer 1 — Durability Core                                      │
│  plans · tasks · evidence · agents · audit_log (hash-chained)   │
│  gate pipeline (identity → ... → validator)                     │
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
| `convergio-db` | 0 | `Pool`, `Migrator`, `Backend` enum | no |
| `convergio-durability` | 1 | `PlanStore`, `TaskStore`, `EvidenceStore`, `AuditLog`, `GatePipeline` | yes (`plans`, `tasks`, `evidence`, `agents`, `audit_log`) |
| `convergio-bus` | 2 | `Bus::publish`, `Bus::subscribe` | yes (`agent_messages`) |
| `convergio-lifecycle` | 3 | `Supervisor::spawn`, `Reaper::tick` | yes (`agent_processes`) |
| `convergio-server` | shell | `app::Router::router()` | no |
| `convergio-planner` | 4 | `solve(mission) -> Plan` | no |
| `convergio-thor` | 4 | `validate(plan_id) -> Verdict` | no |
| `convergio-executor` | 4 | `Executor::tick` | no |
| `convergio-worktree` | 4 | `Worktree::create`, `Worktree::cleanup` | no |
| `convergio-cli` | 4 | `cvg` binary | no |

## Request lifecycle (Layer 1, happy path)

```
client (HTTP)
   │
   ▼
convergio-server (axum router)
   │ extracts path, body, auth context
   ▼
convergio-durability::api
   │ runs gate pipeline against current DB state
   ▼
convergio-db::Pool
   │ executes the transition + audit_log write in one transaction
   ▼
response (HTTP) ──▶ client
```

A failed gate returns `409 Conflict` with a structured body explaining
which gate refused and why.

## Modes (single binary)

```rust
match config.db.scheme() {
    "sqlite" => personal_setup(),
    "postgres" => team_setup(),
}
```

- `personal_setup`: SQLite at `~/.convergio/state.db`,
  no auth (localhost bypass), single implicit `org_id = "default"`.
- `team_setup`: Postgres URL, HMAC auth middleware required,
  `org_id` extracted from request header / token.

The match happens in three places: pool init, migration directory choice,
auth middleware wiring. Everything else is mode-agnostic.

## Audit hash chain

Every row in `audit_log` is:

```text
{
  id: uuid,
  prev_hash: hex,           // hash of the previous row (or genesis "0..0")
  payload: canonical_json,  // entity_type, entity_id, transition, agent, ts, ...
  hash: sha256(prev_hash || payload)
}
```

`GET /v1/audit/verify?from=<id>&to=<id>` recomputes the chain from
the requested range. An external cron can call this hourly.

Hash is computed over **canonical** JSON (sorted keys, no whitespace) to
avoid false positives from formatting drift between clients.

## Background loops

There is **one** background loop in Layer 1, owned by the durability crate:

- **Reaper** (every 60s): tasks `in_progress` whose agent's last heartbeat
  is older than `agent_timeout_seconds` get released back to `pending`,
  the agent gets marked `unhealthy`. An audit row is written for each
  state change.

Layer 4 may add additional loops (planner refresher, executor dispatcher),
but they live in the Layer 4 crates and are optional.

## Where to look for things

- HTTP routes → `crates/convergio-server/src/routes/`
- Schema → `crates/convergio-durability/migrations/`
- Gate pipeline → `crates/convergio-durability/src/gates/`
- Audit verifier → `crates/convergio-durability/src/audit.rs`
- CLI commands → `crates/convergio-cli/src/commands/`
- E2E tests → `tests/`
