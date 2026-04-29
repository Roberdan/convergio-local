# Agent protocol

Agent hosts should use Convergio through MCP. The bridge intentionally
exposes exactly two tools:

| Tool | Purpose |
|------|---------|
| `convergio.help` | read the stable schema, prompt, setup notes, and action catalog |
| `convergio.act` | execute one typed action from the closed action enum |

Agents should call `convergio.help` once per session, then call
`convergio.act` with:

```json
{
  "schema_version": "1",
  "action": "status",
  "params": {}
}
```

Supported actions are:

`status`, `create_plan`, `create_task`, `list_tasks`, `next_task`,
`claim_task`, `heartbeat`, `add_evidence`, `submit_task`,
`complete_task`, `validate_plan`, `audit_verify`,
`explain_last_refusal`, and `agent_prompt`.

## Required loop

1. Call `status`.
2. Create or receive a plan/task.
3. Claim a task with `claim_task`.
4. Add evidence with `add_evidence`.
5. Submit with `submit_task`.
6. If the response code is `gate_refused`, fix the issue, add new
   evidence, and retry `submit_task`.
7. Only report completion after `submit_task` or `complete_task`
   succeeds.
8. Verify with `audit_verify` when closing important work.

`convergio.act` is not a raw HTTP proxy. New behavior must be added as a
new typed action so agent prompts stay small and stable.

`explain_last_refusal` reads the latest durable `task.refused` audit row
when the daemon is reachable, so an agent can recover context even after
the MCP bridge restarts.
