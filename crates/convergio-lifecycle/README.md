# convergio-lifecycle

Layer 3 of Convergio — agent process supervision.

**Status: skeleton.** See [ROADMAP.md](../../ROADMAP.md) week 3-4 for
the intended scope.

## Planned

- `Supervisor::spawn(command, env, plan_id, task_id)` — launches a
  process and persists `agent_processes` row.
- `Supervisor::heartbeat(agent_id)` — keep-alive endpoint.
- `Reaper::tick()` — every 60s, releases tasks whose agent's heartbeat
  is older than `agent_timeout_seconds`.

## Use case

Plan with a 6-hour critical task. Agent's context window dies after
2 hours. With Layer 3 the reaper notices in 60s, the task moves back
to `pending`, an audit row is written, and a new agent is spawned to
pick up where the previous one left off.
