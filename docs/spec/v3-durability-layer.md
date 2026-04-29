# Convergio v3 — local durability runtime spec

Status: current local-first scope.

## Vision

Convergio is a local SQLite-backed runtime for AI-agent work. It gives
any local agent runner durable state, evidence-gated task transitions,
a hash-chained audit log, a small message bus and process supervision.

It does not replace the agent framework. It is the leash underneath it.

## Non-negotiable scope

1. **Local SQLite only.** One daemon, one user, one SQLite file.
2. **Cooperate, don't compete.** Existing agent frameworks are clients.
3. **Reference implementation ships.** Planner, executor and Thor prove
   the loop without becoming a large framework.
4. **Anti-feature creep.** No hosted service, account model, RBAC,
   marketplace or distributed runtime in the MVP.
5. **Every feature must be explainable quickly.** If a feature needs a
   platform diagram to justify it, it is probably not local-MVP scope.

## Layer 1 — Durability Core

Persistent state + gates + audit.

Tables:

- `plans`
- `tasks`
- `evidence`
- `agents`
- `audit_log`

Core API:

```text
POST /v1/plans
GET  /v1/plans
GET  /v1/plans/:id
POST /v1/plans/:id/tasks
GET  /v1/plans/:id/tasks
GET  /v1/tasks/:id
POST /v1/tasks/:id/transition
POST /v1/tasks/:id/evidence
POST /v1/tasks/:id/heartbeat
GET  /v1/audit/verify
```

Guarantees:

- state survives process restart because it is persisted in SQLite
- task transitions run server-side gates
- audited events are hash-chained and locally verifiable
- stale in-progress tasks can be released by the reaper

## Layer 2 — Agent Message Bus

Small persistent message bus scoped per plan.

API:

```text
POST /v1/plans/:id/messages
GET  /v1/plans/:id/messages?topic=&cursor=&limit=
POST /v1/messages/:id/ack
```

Semantics:

- persistent rows in SQLite
- per-topic cursor polling
- acked messages are hidden from future polls
- consumers must be idempotent

## Layer 3 — Agent Lifecycle

Local process tracking for agent workers.

API:

```text
POST /v1/agents/spawn
GET  /v1/agents/:id
POST /v1/agents/:id/heartbeat
```

Important: spawned processes run with the daemon user's privileges. This
is not a sandbox.

## Layer 4 — Reference local flow

Reference components built on layers 1-3:

- `Planner::solve`: creates a plan from a newline-separated mission
- `Executor::tick`: dispatches ready tasks by spawning local workers
- `Thor::validate`: validates completed tasks before close
- `cvg`: pure HTTP CLI client

The reference flow is intentionally small. Users can replace it with
their own client while keeping the local runtime.

## Deployment model

```bash
cargo install --path crates/convergio-server
cargo install --path crates/convergio-cli
convergio start
cvg health
```

Defaults:

```text
database: sqlite://$HOME/.convergio/state.db?mode=rwc
bind:     127.0.0.1:8420
```

## Technology choices

- Rust
- axum
- tokio
- sqlx + SQLite
- clap
- tracing
- Fluent for CLI i18n

## Explicit non-goals

- remote service
- account or organization model
- RBAC
- distributed scheduling
- graphical UI
- marketplace or skill registry
- custom migration framework

## Immediate hardening tasks

- add CLI output modes (`human`, `json`, `plain`)
- replace the deterministic reference executor with a practical local
  adapter for one real agent runner
- add packaged release artifacts beyond `cargo install --path`
