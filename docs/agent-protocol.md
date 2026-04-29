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

For the full swarm model, see
[multi-agent-operating-model.md](./multi-agent-operating-model.md).

Supported actions are:

`status`, `create_plan`, `create_task`, `list_tasks`, `next_task`,
`claim_task`, `heartbeat`, `add_evidence`, `submit_task`,
`complete_task`, `validate_plan`, `audit_verify`,
`import_crdt_ops`, `list_crdt_conflicts`, `explain_last_refusal`, and
`agent_prompt`, plus workspace lease actions:
`claim_workspace_lease`, `list_workspace_leases`,
`release_workspace_lease`, `submit_patch_proposal`, and
`list_workspace_conflicts`.

## Required loop

1. Call `status` to read daemon health, active plans and recent work.
2. Use a unique `agent_id` for this running session.
3. Create or receive a plan/task.
4. Claim a task with `claim_task`.
5. Before mutating workspace files, claim a matching resource lease with
   `claim_workspace_lease`.
6. Send heartbeat while working.
7. Add evidence with `add_evidence`.
8. Submit file changes as a patch proposal with `submit_patch_proposal`
   while the matching leases are still active.
9. Release workspace leases after the proposal is accepted or the work is
   abandoned.
10. Submit with `submit_task`.
11. If the response code is `gate_refused`, fix the issue, add new
   evidence, and retry `submit_task`. For `crdt_conflict` refusals,
   inspect `list_crdt_conflicts`, resolve the conflicting field through a
   new CRDT operation, then retry.
12. Only report completion after `submit_task` or `complete_task`
   succeeds.
13. Verify with `audit_verify` when closing important work.

`convergio.act` is not a raw HTTP proxy. New behavior must be added as a
new typed action so agent prompts stay small and stable.

`explain_last_refusal` reads the latest durable `task.refused` audit row
when the daemon is reachable, so an agent can recover context even after
the MCP bridge restarts.
