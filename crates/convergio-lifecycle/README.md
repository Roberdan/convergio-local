# convergio-lifecycle

Layer 3 of Convergio — agent process supervision.

## Status

**Implemented (basic).** Spawn + persist row + heartbeat + mark-exited
work end-to-end. The supervisor records spawn failures as `failed` rows,
returns a specific error for invalid persisted timestamps, and bounds the
async bookkeeping around spawn with a default timeout.

The watcher loop detects unexpected exits on POSIX platforms with
`kill -0`. Windows PID probing is intentionally unsupported in the MVP;
on Windows the watcher leaves rows as `running` until a platform-specific
probe is implemented.

## API

| Op | Function |
|----|----------|
| Spawn | `Supervisor::spawn(SpawnSpec { kind, command, args, env, plan_id, task_id })` |
| Spawn with timeout | `Supervisor::spawn_with_timeout(spec, duration)` |
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
