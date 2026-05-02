---
id: 0027
status: accepted
date: 2026-05-02
topics: [layer4, executor, dispatch, daemon]
related_adrs: [0009, 0026]
touches_crates: [convergio-server, convergio-executor]
last_validated: 2026-05-02
---

# 0027. Wire the Layer 4 executor loop in the daemon

- Status: accepted
- Date: 2026-05-02
- Deciders: Roberdan
- Tags: layer4, executor, daemon, dispatch

## Context and Problem Statement

Until v0.3.0 the daemon ran two background loops — `Reaper`
(`convergio_durability::reaper`) and `Watcher`
(`convergio_lifecycle::watcher`). The Layer 4 executor
(`convergio_executor::spawn_loop`) was implemented and unit-tested,
but **not wired** from `crates/convergio-server/src/main.rs`. Dispatch
was only available as a one-shot HTTP request: `POST /v1/dispatch`
runs exactly one `Executor::tick()` and returns. AGENTS.md was
explicit about the gap: *"Wire it when you're ready (and document
the reason in an ADR)."*

The cost of that gap: a `pending` task with a satisfied wave
sequence sat there until a human (or `cvg dispatch`) prodded it.
The daemon was not autonomous about its own ready work — it
required external poking — which contradicts the "leash that
moves agents through their own queue" framing in
[ADR-0009](./0009-runner-adapters.md).

## Decision Drivers

- **Autonomy.** A locally-running daemon should pick up ready work
  on its own, with zero external prodding.
- **Symmetry with reaper / watcher.** Both already run as
  background loops with `CONVERGIO_*_TICK_SECS` env-var knobs.
  The executor is the natural third member of that family.
- **No new safety surface.** The HTTP `POST /v1/dispatch` already
  performed the same `Executor::tick()` call. Wiring it on a timer
  is purely additive — same code path, same DB transactions, same
  audit rows.
- **Honest dual entry-point.** Operations/tests still need a manual
  tick. Keep `POST /v1/dispatch` as the manual entry-point, run the
  loop on top.

## Considered Options

1. **Wire the loop, keep `POST /v1/dispatch` (chosen).** Mirror the
   reaper/watcher pattern in `main.rs::start`. Default tick 30s via
   `CONVERGIO_EXECUTOR_TICK_SECS`. The HTTP endpoint stays as a
   manual override and a test seam. Concurrent ticks are safe:
   SQLite serialises writes, and `Durability::transition_task` is
   idempotent (a task already promoted to `in_progress` will be
   rejected by the gate, the loop logs and moves on).
2. **Replace `POST /v1/dispatch` with the loop.** Removing the HTTP
   endpoint would break tests, smoke scripts, and the CLI surface
   (`cvg dispatch`). Rejected.
3. **Wire the loop only behind a feature flag.** Adds a dimension we
   would never want off in production. Rejected.

## Decision Outcome

Chosen option **(1): Wire `executor_spawn_loop` in `main.rs::start`
alongside reaper and watcher**, keep `POST /v1/dispatch` and
`cvg dispatch` available.

The loop is built once at boot from the same `Durability`,
`Supervisor`, and `SpawnTemplate::default()` that the HTTP route
constructs per-request. Tick interval is configurable via
`CONVERGIO_EXECUTOR_TICK_SECS`, default 30 seconds — same default
as the watcher. The handle is dropped (fire-and-forget) like the
reaper and watcher: the loop survives tick failures and only ends
when the process exits.

## Consequences

- **Positive.** A pending task in a satisfied wave is dispatched
  within one tick of becoming ready. CLI users do not need to poll
  `cvg dispatch`. The daemon shape now matches its own
  documentation.
- **Positive.** No new public API. No schema change. No new
  dependency. Only main.rs wiring + ADR + AGENTS.md update.
- **Negative.** A misbehaving template (e.g. a `command` that
  crash-loops) will now crash-loop on a 30-second cadence on
  startup, instead of staying dormant until prodded. This is the
  *correct* failure mode (loud, observable) but operators should
  know about it. The watcher will still tick exited processes
  toward `exited` status; the reaper still cleans heart-beat-stale
  tasks; the audit chain captures every transition.
- **Negative.** Two dispatchers (loop + HTTP) on the same task can
  race. The race is benign — one wins, the other gets a gate
  refusal — but it adds rows to the audit log on contention. If
  this becomes noisy we add an explicit claim lock (future ADR);
  for now we accept it.

## Validation

- `cargo check -p convergio-server` clean after the wiring.
- Existing executor unit tests in
  `crates/convergio-executor/tests/dispatch.rs` keep passing.
- New integration test boots the in-process server, creates a
  one-task plan with no wave dependencies, waits one tick, asserts
  the task moved to `in_progress` without any `POST /v1/dispatch`
  call.
- `cvg dispatch` still works (it uses the HTTP endpoint, untouched).

## Knobs

| Env var | Default | Effect |
|---|---|---|
| `CONVERGIO_EXECUTOR_TICK_SECS` | `30` | Seconds between consecutive `Executor::tick()` runs. Set high to make the daemon nearly idle for tests. Set to a small value to chase responsive dispatch. |

## Out of scope

- Smarter task selection (priority, deadline, fairness). The MVP
  loop calls `tick()` which already dispatches all wave-ready tasks.
- A claim lock that prevents the loop and HTTP dispatch from
  racing. SQLite serialisation plus gate-refusal idempotency is
  enough for now.
- Multiple concurrent executors. There is exactly one loop per
  daemon process; one daemon per host.
