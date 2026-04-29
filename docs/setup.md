# Local setup

Convergio v3 is a single-user local daemon. It needs no account, no
Postgres, and no external service.

```bash
sh scripts/install-local.sh
cvg setup
convergio start
```

In another terminal:

```bash
cvg doctor
cvg demo
```

To install the daemon as a user-level service:

```bash
cvg service install
cvg service start
cvg service status
```

The default state lives under `~/.convergio/`:

| Path | Purpose |
|------|---------|
| `config.toml` | local URL, bind address, SQLite URL |
| `v3/state.db` | SQLite database |
| `daemon.pid` | daemon discovery for `cvg doctor` |
| `adapters/` | generated agent snippets |
| `mcp.log` | compact MCP action diagnostics |

## Release artifacts

CI builds unsigned Linux and macOS tarballs on release tags. Locally, you
can produce the same shape with:

```bash
sh scripts/package-local.sh
```

macOS signing and notarization are intentionally not faked in this repo;
release artifacts are unsigned until real signing credentials exist.

## Agent setup

Generate host-specific snippets:

```bash
cvg setup agent claude
cvg setup agent cursor
cvg setup agent qwen
```

Each command writes `mcp.json`, `prompt.txt`, and `README.txt` under
`~/.convergio/adapters/<host>/`. Copy `mcp.json` into the host's MCP
configuration and `prompt.txt` into its custom instructions.

All snippets use the same bridge:

```bash
convergio-mcp --url http://127.0.0.1:8420
```

If an agent cannot connect, run:

```bash
cvg doctor --json
cvg mcp tail
```
