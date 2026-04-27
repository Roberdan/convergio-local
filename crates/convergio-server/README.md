# convergio-server

HTTP routing shell for Convergio. Hosts:

- Layer 1 API (`/v1/plans`, `/v1/tasks`, `/v1/audit/verify`)
- (future) Layer 2 message-bus endpoints
- (future) MCP surface

The server crate is a **shell** — all logic lives in
`convergio-durability` (and, later, `convergio-bus` and
`convergio-lifecycle`).

## Run

```bash
cargo run -p convergio-server
# default mode: SQLite at $HOME/.convergio/state.db, listens on 127.0.0.1:8420
```

Environment:

| Variable | Default | Notes |
|----------|---------|-------|
| `CONVERGIO_DB` | `sqlite://$HOME/.convergio/state.db?mode=rwc` | `postgres://...` for team mode |
| `CONVERGIO_BIND` | `127.0.0.1:8420` | TCP socket |
| `CONVERGIO_LOG` | `info` | `tracing-subscriber` filter |

## API surface

| Method | Path | What it does |
|--------|------|--------------|
| `GET`  | `/v1/health` | Liveness + version |
| `POST` | `/v1/plans` | Create a plan |
| `GET`  | `/v1/plans` | List plans (`?org_id=&limit=`) |
| `GET`  | `/v1/plans/:id` | Get one plan |
| `POST` | `/v1/plans/:plan_id/tasks` | Create a task |
| `GET`  | `/v1/plans/:plan_id/tasks` | List tasks of a plan |
| `GET`  | `/v1/tasks/:id` | Get one task |
| `POST` | `/v1/tasks/:id/transition` | `{ target, agent_id? }` — runs gate pipeline |
| `POST` | `/v1/tasks/:id/heartbeat` | Touch heartbeat |
| `POST` | `/v1/tasks/:id/evidence` | `{ kind, payload, exit_code? }` |
| `GET`  | `/v1/tasks/:id/evidence` | List evidence |
| `GET`  | `/v1/audit/verify` | Recompute hash chain (`?from=&to=`) |

Errors are JSON: `{ "error": { "code", "message" } }`. `409 gate_refused`
when a gate refuses; `404 not_found` for missing entities.
