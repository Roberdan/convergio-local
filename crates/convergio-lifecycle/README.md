# convergio-lifecycle

Layer 3 of Convergio — agent process supervision.

## Status

**Implemented (basic).** Spawn + persist row + heartbeat + mark-exited
work end-to-end. Watcher loop that detects unexpected exits is not
yet wired (planned Layer 3 follow-up).

## API

| Op | Function |
|----|----------|
| Spawn | `Supervisor::spawn(SpawnSpec { kind, command, args, env, plan_id, task_id })` |
| Get | `Supervisor::get(id)` |
| Heartbeat | `Supervisor::heartbeat(id)` |
| Mark exited | `Supervisor::mark_exited(id, exit_code, ok)` |

HTTP surface (mounted by `convergio-server`):

| Method | Path |
|--------|------|
| `POST` | `/v1/agents/spawn` |
| `GET`  | `/v1/agents/:id` |
| `POST` | `/v1/agents/:id/heartbeat` |

## Use case

Plan with a 6-hour critical task. Agent's context window dies after
2 hours. With Layer 3 the daemon notices the missing heartbeat in 60s,
the Layer 1 reaper releases the task back to `pending`, an audit row
is written, and the executor (Layer 4) picks it up with a new agent.

## What it is NOT

- **Not** systemd / launchd — we don't manage system services.
- **Not** a sandbox — agents run with the daemon's privileges.
- **Not** Kubernetes — no resource limits, no scheduling, no networking.
