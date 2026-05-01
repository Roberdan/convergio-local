# `cvg-attach` — Convergio session attach skill

A reference skill that registers a Claude Code session with the
local Convergio daemon so peer sessions can see each other through
`cvg status --agents` and the `system.session-events` bus topic.

## What problem this solves

Two Claude Code sessions running concurrently in the same repo
have no native way to know about each other. Each is its own
process tree, with its own context window, editing the same
codebase. The classic failure mode (observed 2026-05-01 on this
very repo): two sessions edit overlapping files in parallel, the
operator only learns about the conflict when one of the two
commits and the other gets a merge surprise.

Convergio's agent registry + bus exist exactly to close this gap,
but only if the agent host (Claude Code, Copilot CLI, Cursor, …)
actually calls them. This skill is the calling code for Claude
Code.

## Files in this directory

| File | Role |
|---|---|
| `SKILL.md` | Skill manifest with frontmatter and trigger phrases |
| `cvg-attach.sh` | Bash runner that POSTs to the agent registry and the bus |
| `README.md` | This file |

## Installation

### Via the `cvg setup agent claude` installer (recommended)

```bash
cvg setup agent claude
```

The installer copies the skill into `~/.claude/skills/cvg-attach/`
and writes a matching `.claude/settings.json` template that wires
the SessionStart hook to invoke the skill automatically.

### Manual installation

```bash
mkdir -p ~/.claude/skills/cvg-attach
cp examples/skills/cvg-attach/* ~/.claude/skills/cvg-attach/
chmod +x ~/.claude/skills/cvg-attach/cvg-attach.sh
```

Then add to `~/.claude/settings.json` (or the per-repo
`.claude/settings.json`):

```json
{
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "bash ~/.claude/skills/cvg-attach/cvg-attach.sh",
            "timeout": 5,
            "async": true
          }
        ]
      }
    ]
  }
}
```

## Reproducing the failure mode

Without the skill installed:

```bash
# Terminal 1
cd ~/your/repo
claude

# Terminal 2 (in parallel)
cd ~/your/repo
claude

# In either session
cvg status --agents
# → "Active agents: (none)" — both sessions are invisible to the daemon
```

With the skill installed and the daemon running:

```bash
cvg setup agent claude        # one-time setup
# ... new sessions automatically register at start

# Open two sessions as before, then in either:
cvg status --agents
# → both sessions visible with id, kind, host, last heartbeat,
#   held leases, current task
```

## What gets written to the daemon

On a successful registration the skill produces, in order:

1. One row in `agent_registry.agents` (the durable identity).
2. One audit-log row of kind `agent.registered`.
3. One `system.session-events` bus message of kind `agent.attached`.

All three are visible via:

```bash
cvg status --agents                            # the registry view
cvg audit verify --range last-1h               # the audit chain
curl -s "$CVG/v1/system-messages?topic=system.session-events&limit=10"
```

## Degraded modes

| Condition | Behaviour | Audit row? |
|---|---|---|
| Daemon offline at session start | Warning on stderr, skill exits 0 | No |
| Registration succeeds but bus publish fails | Skill exits 0; presence missing but identity is durable | `agent.registered` only |
| Registration fails after 3 retries | Warning on stderr, skill exits 0 | No |

The skill **never blocks the user's work**. A coordinated session
is preferable but the user can always proceed without it; that is
the explicit design choice in PRD-001 § Risks.

## See also

- ADR-0023 — `system.session-events` topic family with
  `plan_id IS NULL` (`docs/adr/0023-system-session-events-topic.md`)
- PRD-001 — Claude Code adapter
  (`docs/prd/0001-claude-code-adapter.md`)
- Adversarial review of PRD-001 v1
  (`docs/reviews/PRD-001-adversarial-review-v1.md`)
- The companion installer: `cvg setup agent claude`
- The visibility surface: `cvg status --agents`
- The end-of-session safety net: `cvg session pre-stop`
