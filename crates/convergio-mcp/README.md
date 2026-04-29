# convergio-mcp

Stdio MCP bridge for the local Convergio daemon.

It exposes exactly two tools:

- `convergio.help`
- `convergio.act`

The bridge contains no gate or persistence logic. It forwards typed
actions to the local HTTP daemon and returns compact structured JSON for
agents.
