---
name: cvg-attach
version: 1.0.0
description: |
  Register the current Claude Code session as a Convergio agent so the
  local daemon can see who is online, broadcast presence on the
  `system.session-events` bus, and coordinate with peer sessions in
  the same repo. Invoke once at session start; the heartbeat and
  detach flow are wired by the companion `.claude/settings.json`
  hooks (see PRD-001 Artefact 2).
allowed-tools:
  - Bash
triggers:
  - cvg attach
  - register session
  - convergio attach
preamble-tier: 4
---

# /cvg-attach — register this Claude Code session with Convergio

## What this skill does

1. Generates a stable session-local agent id of the form
   `claude-code-${USER}-${HEX8}`.
2. POSTs to `http://127.0.0.1:8420/v1/agent-registry/agents` with the
   `NewAgent` payload (kind, name, host, capabilities, metadata).
3. On success, persists the returned `agent_id` to
   `~/.convergio/state/sessions/${PID}.agent` so subsequent hooks
   (PreToolUse, Stop) can read it.
4. Publishes one `agent.attached` message on the system-scoped
   `system.session-events` bus topic (introduced by ADR-0023, allowed
   to have `plan_id IS NULL`).
5. Prints a one-line confirmation: `Convergio agent registered: <id>`.

## When to invoke

Once at session start (the `.claude/settings.json` `SessionStart`
hook installed by `cvg setup agent claude` does this automatically).
Manually invoke with `/cvg-attach` only when the daemon was started
mid-session and you want the current session to appear in
`cvg status --agents` retroactively.

## Pre-flight

The daemon must be reachable at `${CONVERGIO_API_URL:-http://127.0.0.1:8420}`.
If the daemon is offline, the skill prints a single warning and
exits 0 — it does **not** block the user's work (PRD-001 § Risks).

## Execution

```bash
bash "$(dirname "$0")/cvg-attach.sh"
```

The script runs the full flow with idempotent retries (max 3
attempts, 500 ms backoff) and prints the agent id on success.

## Output contract

| Stream | Content |
|---|---|
| stdout | `Convergio agent registered: <agent_id>` on success, nothing on no-op |
| stderr | One-line warning when daemon unreachable; nothing otherwise |
| exit | `0` always (failures are surfaced via stderr, never block) |
| filesystem | `~/.convergio/state/sessions/${PID}.agent` written on success |

## Relation to the rest of Wave 0b

- The bus topic `system.session-events` is defined in ADR-0023 and
  the schema migration ships in `0103_system_topics.sql`.
- The companion hooks (`PreToolUse`, `Stop`) live in the
  `.claude/settings.json` template emitted by `cvg setup agent
  claude` (Wave 0b task w1.5).
- The visibility surface is `cvg status --agents` (Wave 0b task
  w1.6).
- The end-of-session safety net is `cvg session pre-stop` (Wave 0b
  task w1.4b).

## Why this is a skill, not a subcommand

The agent host (Claude Code) launches skills with full session
context (env, working directory, credentials). A `cvg attach`
subcommand would have to re-discover the host context from outside.
Keeping the registration in a skill means the agent itself is the
registrant, not a human-typed command, which is what the audit log
should record.
