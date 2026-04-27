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

| Method | Path | What it does | Layer |
|--------|------|--------------|-------|
| `GET`  | `/v1/health` | Liveness + version | shell |
| `POST` | `/v1/plans` | Create a plan | 1 |
| `GET`  | `/v1/plans` | List plans (`?org_id=&limit=`) | 1 |
| `GET`  | `/v1/plans/:id` | Get one plan | 1 |
| `POST` | `/v1/plans/:plan_id/tasks` | Create a task | 1 |
| `GET`  | `/v1/plans/:plan_id/tasks` | List tasks of a plan | 1 |
| `GET`  | `/v1/tasks/:id` | Get one task | 1 |
| `POST` | `/v1/tasks/:id/transition` | `{ target, agent_id? }` — runs gate pipeline | 1 |
| `POST` | `/v1/tasks/:id/heartbeat` | Touch heartbeat | 1 |
| `POST` | `/v1/tasks/:id/evidence` | `{ kind, payload, exit_code? }` | 1 |
| `GET`  | `/v1/tasks/:id/evidence` | List evidence | 1 |
| `GET`  | `/v1/audit/verify` | Recompute hash chain (`?from=&to=`) | 1 |
| `POST` | `/v1/plans/:plan_id/messages` | Publish on the bus | 2 |
| `GET`  | `/v1/plans/:plan_id/messages` | Poll (`?topic=&cursor=&limit=`) | 2 |
| `POST` | `/v1/messages/:id/ack` | Consumer ack | 2 |
| `POST` | `/v1/agents/spawn` | Spawn a tracked process | 3 |
| `GET`  | `/v1/agents/:id` | Get process row | 3 |
| `POST` | `/v1/agents/:id/heartbeat` | Touch process heartbeat | 3 |

Errors are JSON: `{ "error": { "code", "message" } }`.

| Status | Code | When |
|--------|------|------|
| 404 | `not_found` | missing plan / task / evidence / message / agent |
| 409 | `gate_refused` | Layer 1 gate refused a task transition |
| 422 | `spawn_failed` | Layer 3 could not `exec` the requested binary |
| 500 | `audit_broken` / `internal` | server-side fault |
