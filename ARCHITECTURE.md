# Architecture

Convergio is a local-first runtime. One daemon runs on the user's
machine, persists state in one SQLite database, and exposes a localhost
HTTP API for agents and the `cvg` CLI.

## The layers

```text
┌──────────────────────────────────────────────────────────────┐
│  Layer 4 — Reference Local Flow                              │
│  planner · thor validator · executor · CLI: cvg              │
├──────────────────────────────────────────────────────────────┤
│  Layer 3 — Agent Lifecycle                                   │
│  spawn · heartbeat · process watcher                         │
├──────────────────────────────────────────────────────────────┤
│  Layer 2 — Agent Message Bus                                 │
│  publish · poll by cursor · ack · scoped per plan            │
├──────────────────────────────────────────────────────────────┤
│  Layer 1 — Durability Core                                   │
│  plans · tasks · evidence · audit_log · gates · reaper       │
├──────────────────────────────────────────────────────────────┤
│  convergio-db                                                │
│  SQLite pool + per-crate migrations                          │
└──────────────────────────────────────────────────────────────┘
```

Lower layers do not depend on higher layers. A custom client can use
Layer 1-3 directly and ignore the reference Layer 4 crates.

## Crate map

| Crate | Layer | Public surface | Owns DB tables? |
|-------|-------|----------------|-----------------|
| `convergio-db` | 0 | `Pool`, `Backend` | no |
| `convergio-durability` | 1 | `Durability`, stores, audit, gates, reaper | yes (`plans`, `tasks`, `evidence`, `agents`, `audit_log`) |
| `convergio-bus` | 2 | `Bus::publish`, `Bus::poll`, `Bus::ack` | yes (`agent_messages`) |
| `convergio-lifecycle` | 3 | `Supervisor::spawn`, `heartbeat`, `mark_exited`, `get`, watcher | yes (`agent_processes`) |
| `convergio-server` | shell | `router(state)`, `AppState`, `convergio start` | no |
| `convergio-cli` | 4 | `cvg` binary | no |
| `convergio-planner` | 4 | `Planner::solve` | no |
| `convergio-thor` | 4 | `Thor::validate` -> `Verdict` | no |
| `convergio-executor` | 4 | `Executor::tick`, `spawn_loop` | no |

## HTTP surface

All endpoints sit under `/v1`. Errors are:

```json
{ "error": { "code": "not_found", "message": "..." } }
```

| Status | Code | When |
|--------|------|------|
| 404 | `not_found` | missing plan / task / evidence / message / agent |
| 409 | `gate_refused` | a gate refused a task transition |
| 422 | `spawn_failed` | Layer 3 could not execute the requested binary |
| 500 | `audit_broken` / `internal` | server-side fault |

| Method | Path | Layer |
|--------|------|-------|
| GET | `/v1/health` | shell |
| POST / GET | `/v1/plans` | 1 |
| GET | `/v1/plans/:id` | 1 |
| POST / GET | `/v1/plans/:plan_id/tasks` | 1 |
| GET | `/v1/tasks/:id` | 1 |
| POST | `/v1/tasks/:id/transition` | 1 |
| POST | `/v1/tasks/:id/heartbeat` | 1 |
| POST / GET | `/v1/tasks/:id/evidence` | 1 |
| GET | `/v1/audit/verify` | 1 |
| POST / GET | `/v1/plans/:plan_id/messages` | 2 |
| POST | `/v1/messages/:id/ack` | 2 |
| POST | `/v1/agents/spawn` | 3 |
| GET | `/v1/agents/:id` | 3 |
| POST | `/v1/agents/:id/heartbeat` | 3 |
| POST | `/v1/solve` | 4 |
| POST | `/v1/dispatch` | 4 |
| POST | `/v1/plans/:id/validate` | 4 |

## Request lifecycle: task transition

```text
client POST /v1/tasks/:id/transition
   |
   v
convergio-server handler
   |
   v
convergio-durability::Durability::transition_task
   |
   |- runs gate pipeline
   |- on refusal: HTTP 409 gate_refused
   |
   v
updates task state and appends an audit row
   |
   v
response: 200 Task
```

## Local runtime

The daemon defaults to:

```text
database: sqlite://$HOME/.convergio/v3/state.db?mode=rwc
bind:     127.0.0.1:8420
```

Configuration:

| Variable / flag | Default | Notes |
|-----------------|---------|-------|
| `CONVERGIO_DB` / `--db` | local SQLite file | must be `sqlite://...` |
| `CONVERGIO_BIND` / `--bind` | `127.0.0.1:8420` | keep localhost for the local security model |
| `CONVERGIO_LOG` | `info` | tracing filter |

## Audit hash chain

Every audited row contains:

```text
seq, payload, prev_hash, hash
```

where:

```text
hash = sha256(prev_hash || canonical_json(payload))
```

`GET /v1/audit/verify` recomputes the chain. Run open-ended
verification for the strongest local tamper-evidence guarantee.

## Migration coexistence

Each crate owns its own migration files and shares the same SQLite
`_sqlx_migrations` bookkeeping table by version range:

| Crate | Range |
|-------|-------|
| `convergio-durability` | 1-100 |
| `convergio-bus` | 101-200 |
| `convergio-lifecycle` | 201-300 |

Every migrator uses `set_ignore_missing(true)` so independent crates can
coexist in the same local database file.

## Background loops

- **Reaper** — `convergio_durability::reaper::spawn`. Releases stale
  `in_progress` tasks back to `pending` and writes `task.reaped` audit
  rows.
- **Watcher** — `convergio_lifecycle::watcher::spawn`. Polls tracked
  process rows and flips dead PIDs to `exited`.

Layer 4 has `convergio_executor::spawn_loop`, but the daemon currently
uses manual ticks via `POST /v1/dispatch`.

## Where to look

- HTTP routes -> `crates/convergio-server/src/routes/`
- Schema -> `crates/<crate>/migrations/`
- Gates -> `crates/convergio-durability/src/gates/`
- Audit -> `crates/convergio-durability/src/audit/`
- Reaper -> `crates/convergio-durability/src/reaper.rs`
- Bus -> `crates/convergio-bus/src/bus.rs`
- Supervisor -> `crates/convergio-lifecycle/src/supervisor.rs`
- CLI -> `crates/convergio-cli/src/commands/`
- E2E tests -> `crates/convergio-server/tests/e2e_*.rs`
