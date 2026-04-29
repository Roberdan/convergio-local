# Agent host setup

All supported hosts use the same Convergio MCP bridge:

```bash
convergio-mcp --url http://127.0.0.1:8420
```

Generate the exact snippets for your host:

```bash
cvg setup agent <host>
```

Supported hosts:

| Host | Command |
|------|---------|
| Claude Desktop / Claude Code | `cvg setup agent claude` |
| GitHub Copilot local IDE integrations | `cvg setup agent copilot-local` |
| GitHub Copilot cloud agent repository hint | `cvg setup agent copilot-cloud` |
| Cursor | `cvg setup agent cursor` |
| Cline | `cvg setup agent cline` |
| Continue | `cvg setup agent continue` |
| Qwen / qwen-code | `cvg setup agent qwen` |
| Generic shell agent | `cvg setup agent shell` |

Each generated directory contains:

| File | Use |
|------|-----|
| `mcp.json` | copy into the host MCP configuration |
| `prompt.txt` | copy into custom instructions |
| `README.txt` | host-local reminder |

## Required agent behavior

1. Call `convergio.help` once.
2. Use `convergio.act`; do not call daemon HTTP endpoints directly.
3. Use a unique `agent_id` for each running session.
4. Claim tasks before working.
5. Send heartbeat while working.
6. Attach evidence before submit.
7. If `gate_refused`, fix the root cause, attach new evidence, retry.
8. Only tell the user work is complete after Convergio accepts the task.

For multi-agent usage, do not let agents coordinate through private chat
or side files. They should coordinate through Convergio task state,
evidence, audit, and the plan-scoped message bus. See
`docs/multi-agent-operating-model.md`.

## Troubleshooting

```bash
cvg doctor --json
cvg mcp tail
convergio-mcp --version
```

If `doctor` says the daemon is unreachable:

```bash
cvg service start
# or
convergio start
```
