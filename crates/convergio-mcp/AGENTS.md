# AGENTS.md — convergio-mcp

For repo-wide rules see [../../AGENTS.md](../../AGENTS.md).

This crate is the stdio MCP bridge. It adapts agent tools to the daemon;
it is not an orchestrator and not a raw HTTP proxy.

## Invariants

- Expose only `convergio.help` and `convergio.act`.
- Keep prompts compact; put durable context in Convergio, not in tool
  descriptions.
- All state-changing actions go through the daemon HTTP API.
- Log diagnostics without leaking secrets.
- Capability actions must remain namespaced behind `convergio.act`.
