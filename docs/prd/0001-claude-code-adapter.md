---
id: PRD-001
status: proposed
date: 2026-05-01
wave: 0
related_adrs: [0006, 0007, 0009, 0011, 0012, 0016, 0018]
---

# PRD-001 — Claude Code adapter for Convergio

> *Two Claude Code sessions in the same repo right now cannot see
> each other through Convergio. That is the bug VISION.md exists
> to fix first.*

## Problem

Convergio v0.2.0 shipped:

- Agent registry (`/v1/agents/spawn`, `/v1/agents/:id/heartbeat`,
  watcher loop, reaper) — ADR-0009
- Workspace leases + patch proposals — ADR-0007
- CRDT actor/op store — ADR-0006
- Persistent agent message bus scoped per plan — Layer 2
- Hash-chained audit log — ADR-0002

But **no Claude Code session ever calls any of these endpoints
during normal work**. The primitives are present; the client
adapter that wires them is missing.

Concrete observed failure mode (2026-05-01, dogfood session):
two concurrent Claude Code sessions in the same repo. Neither
was registered in the agent registry. Neither claimed a
workspace lease before editing files. Neither posted a
heartbeat. Neither published a single bus message. The operator
had to ask the second session what it was doing through the
human channel because the system intended to solve this exact
problem could not answer.

This is the gap that breaks the long-tail thesis (ADR-0016): a
shovel that does not coordinate parallel diggers is a single-user
tool. Closing it is Wave 0 of the new ROADMAP.

## Why now

- **The other primitives exist.** This is wiring, not new
  infrastructure. Cost/benefit ratio is unusually favourable.
- **The operator just lived the failure.** Documenting and
  fixing it while the friction is fresh is cheaper than
  reconstructing it later.
- **VISION.md is being written this week.** Wave 0 must include
  a concrete deliverable that demonstrates the urbanism
  primitives in action, not just describes them. Without this
  PRD shipping, VISION reads as marketing.
- **Industry-alignment story needs a working demo.** When
  pitching Convergio as runtime enforcement of widely-adopted
  engineering principles (see ADR-0017), the elevator must stop
  at "and here's two Claude sessions coordinating via the bus,
  with every action audited".

## What we are building

A **Claude Code adapter** with three artefacts:

### Artefact 1 — The skill `/cvg-attach`

A Claude Code skill (Markdown + minimal Bash preamble) that, when
invoked at session start:

1. Calls `POST /v1/agent-registry/agents` (the *registration*
   endpoint per ADR-0009 — distinct from `/v1/agents/spawn`,
   which spawns a daemon-managed runner process) with:
   ```json
   {
     "kind": "claude-code",
     "name": "claude-code-${USER}-${RANDOM_ID}",
     "host": "${HOSTNAME}",
     "actions": ["edit", "read", "shell", "evidence-attach"],
     "metadata": {
       "tty": "${TTY}",
       "pid": "${$}",
       "cwd": "${PWD}",
       "session_started_at": "${ISO_TIMESTAMP}"
     }
   }
   ```
   The field is named `actions` (verbs the agent will perform), not
   `capabilities`, to avoid clashing with the *capability bundle*
   namespace (`azure.*`, `auth.*`, `ui.*`, …) defined in ADR-0008
   and ADR-0018. They are different concepts; the API surface
   should reflect that.
2. Stores the returned `agent_id` in
   `~/.convergio/state/sessions/${PID}.agent`
3. Emits a heartbeat-and-presence message to the daemon's
   **`system.session-events` topic** (a system-scoped, plan-
   independent bus topic introduced for unattached sessions —
   see "Bus topology" below)
4. Prints to the human:
   ```
   Convergio agent registered: agent_id=…
   Use cvg status --agents to see live activity.
   ```

### Artefact 2 — Hooks for SessionStart / PreToolUse / Stop

Configured in `.claude/settings.json` for the Convergio repo
(later, generalised to any repo where the daemon is reachable):

| Hook | Trigger | Action |
|---|---|---|
| `SessionStart` | session boot | run `/cvg-attach` preamble; capture agent_id |
| `PreToolUse(Edit)` | before any file edit | call `claim_workspace_lease` for the file path; if 409, refuse the edit and surface the conflict to the human |
| `PreToolUse(Write)` | before any file write | same as above |
| `PostToolUse(Edit/Write)` | after a successful edit | publish a `task.touched-file` bus message including the file path and a short summary |
| `Notification` | on agent prompt | publish a `system.idle` bus message so peer sessions can see "this session is waiting on the human" |
| `Stop` | session exit | call `retire_agent`; release outstanding leases |

Heartbeat: a background loop in the hook agent runs `POST
/v1/agents/:id/heartbeat` every 30 seconds. If the daemon is
unreachable for more than 90 seconds, the hook surfaces a
warning to the human ("Convergio daemon offline; coordination
disabled until reconnect") but does **not** block the user's work.

> **Wave 0b cut**: the heartbeat loop is *deferred to Wave 0b.2*.
> The v1 cut ships a single `agent.attached` publish at
> SessionStart and relies on the daemon's reaper (60 s tick,
> 300 s timeout) to mark stale sessions as terminated. A
> proper periodic heartbeat hook lands when Claude Code's
> long-running hook story stabilises or when an external
> watcher process (launchd / systemd / a `cvg session keepalive`
> subcommand) becomes the home of the loop.

### Bus topology — `system.session-events`

Today the agent message bus is plan-scoped: every message belongs
to a `plan_id`. A Claude Code session at startup is *not yet
attached to a plan* — it has no plan context until the user picks
one or claims a task. To enable session-presence broadcasts
("I'm here, my last heartbeat is X, I hold leases on Y") for
unattached sessions, we introduce a single system-scoped topic:

- **Topic name**: `system.session-events`
- **Scope**: system, not plan-scoped (`plan_id` nullable on bus
  messages of this topic kind only)
- **Allowed message kinds**: `agent.attached`, `agent.heartbeat`,
  `agent.idle`, `agent.detached`, `agent.lease-claimed`,
  `agent.lease-released`
- **Retention**: 24h ring buffer (consistent with idle-session
  heartbeat semantics; longer retention is roadmap)
- **Audit**: yes — every system topic message lands in the audit
  log just like plan-scoped messages

This is a small but structural change to the bus contract.
Implementing PRD-001 requires the bus schema migration that
allows `plan_id IS NULL` for system topics; that schema change
is documented in
[ADR-0025](../adr/0025-system-session-events-topic.md) (status
`proposed`, drafted alongside this PRD). The migration itself
ships in `crates/convergio-bus/migrations/0103_system_topics.sql`
on this branch.

### Artefact 4 — `cvg session pre-stop` + Stop-hook integration

> *The vigile urbano does not sign the certificate of habitability
> until the building site is clean.* This is the structural antibody
> against the failure mode where an agent declares "day closed, repo
> clean" while plan tasks, friction-log entries, and bus messages
> tell a different story.

> **Wave 0b cut**: Artefact 4 is *deferred to Wave 0b.2* (plan
> task `168e9561`). The session-end hook in this slice retires
> the agent registration but does not yet run the six-check
> safety net described below. The contract below is the v2 cut.

A new `cvg` subcommand and CLI surface, called by the Stop hook
before the session terminates:

```
cvg session pre-stop --agent-id <id> [--force]
```

The command runs six checks and prints a structured report. Default
exit code is 0 if all checks pass, **non-zero if any actionable gap
is found** so the Stop hook can refuse the silent close (the human
is shown the report and can decide whether to address or `--force`
through).

| # | Check | Implementation |
|---|---|---|
| 1 | **Plan-vs-merged-PR drift** | for each plan this agent has touched, query git log since session start for `Tracks: T<id>` lines in merged PRs; flag tasks whose linked PR is merged but state is still `pending`/`submitted`. Suggested action: `cvg pr sync <plan_id>` (shipped in PR #59 / T2.04 — auto-transitions matching pending tasks to `submitted`). |
| 2 | **Bus messages addressed to me, unconsumed** | `poll_messages` filtered to messages with `payload.to_agent == my-id` and `consumed_at IS NULL`. Suggested action: `POST /v1/messages/:id/ack` (HTTP). `cvg bus` ships `tail`/`topics`/`post` (PR #63) but no `ack` wrapper yet — the curl call is the canonical path. |
| 3 | **Bus messages I sent, unconsumed by recipient** | dual of check 2; warns on stale outbound traffic so I can either manually ack-self if obsolete or wait. |
| 4 | **Worktrees I created with no PR open** | parses `git worktree list --porcelain` filtered by author metadata; cross-references `gh pr list --head <branch> --state all`. Flags abandoned worktrees. |
| 5 | **Files declared in last bus handshake `files_about_to_touch` but never committed** | reads my last bus handshake message (if any), diffs declared paths vs `git log --author=me --since=session-start --name-only`. Flags promises-not-kept. |
| 6 | **Friction log entries hinted in commits but never written** | `git log --grep='F[0-9]+' --since=session-start` extracts new finding IDs; checks they appear in `docs/plans/v*-friction-log.md`. Flags missing entries. |

#### Output format

Human (default): a structured report listing each check with its
findings. JSON: same data, machine-readable. Plain: minimal text
for shell pipelines. All three honour `--output` per existing CLI
plumbing.

#### Stop-hook semantics

The Stop hook calls `cvg session pre-stop` with the current agent
id. Three outcomes:

- **All clear** → hook proceeds with `retire_agent` + lease release
  + final `agent.detached` bus message → session exits cleanly.
- **Actionable gaps found, not forced** → hook prints the report,
  prompts the human ("Address now / Skip with reason / Force
  quit"), and conditionally calls `pre-stop --force` only on
  explicit human ack.
- **Daemon unreachable** → hook surfaces a warning but does not
  block exit (the gate is opt-in safety, not a hostage situation).

A new audit row kind `agent.detached_with_known_gaps` is written
when `--force` is used; the row records which checks fired and
the human override reason if provided.

#### Why this lives in PRD-001 and not as a separate ADR

This is the *implementation* of the constitutional reflex described
in CONSTITUTION § Sacred principles ("agents cannot claim done
before evidence agrees") **applied to the agent itself, at session
end**. The principle was always there; the missing piece was the
mechanical check at the boundary. Bundling it into PRD-001 means
the very first Claude Code adapter to ship is also the first agent
that *cannot lie about being done*. That is the demonstrable proof
that the urban code is real.

#### Validation tests for Artefact 4

| Test | Expectation |
|---|---|
| Session ends with no work done | pre-stop returns 0, hook proceeds clean |
| Session ends with 1 plan task `pending` whose `Tracks:` PR is merged | check 1 fires, suggests `cvg pr sync`, Stop hook surfaces |
| Session ends with 1 unconsumed bus message addressed to me | check 2 fires, hook surfaces |
| Session ends with worktree `feat/foo` no PR open | check 4 fires, hook surfaces |
| Session ends with `Tracks: F35` in a commit but `v0.2-friction-log.md` not modified | check 6 fires, hook surfaces |
| Daemon offline | warning surfaced, hook does not block |
| `--force` flag set by human | hook proceeds despite gaps; `agent.detached_with_known_gaps` audit row written |

### Artefact 3 — `cvg status --agents`

A new flag on the existing `cvg status` command that adds a
section:

```
Active agents:
  claude-code-roberdan-7a2f  (claude-code, ttys001, started 13h ago)
    last heartbeat: 8s ago
    holding leases: docs/adr/0017-ise-hve-alignment.md
    current task: 7a0671b5… (Tier 2 frontmatter on every ADR)
  claude-code-roberdan-9c4d  (claude-code, ttys004, started 44m ago)
    last heartbeat: 2s ago
    holding leases: VISION.md, docs/adr/0016-…
    current task: (no task claimed)

Recent bus activity (last 5 messages):
  …
```

JSON and plain output formats are provided per existing
`--output` plumbing.

## Definition of done

- `cvg-attach.md` ships in `examples/skills/` with installation
  instructions for Claude Code (`~/.claude/skills/`), Cursor,
  and Codex CLI.
- `.claude/settings.json` template at repo root showing how to
  wire the four hook events.
- `cvg status --agents` ships in `convergio-cli` and is covered
  by an E2E test that boots two ephemeral agent registrations
  and verifies they appear with correct metadata.
- An audit row exists for every agent registration, heartbeat
  gap, lease claim, lease release, and retirement. Verified by
  `cvg audit verify --range last-1h`.
- A README section in `examples/skills/cvg-attach/README.md`
  reproduces the failure mode this PRD opens with (two sessions
  in the same repo) and shows the `cvg status --agents` output
  with both visible.

## Validation tests

| Test | Expectation |
|---|---|
| Boot two `claude` processes in the repo with the skill installed | Both appear in `cvg status --agents` within 30s |
| One session calls `Edit` on file X while the other holds a lease on X | Edit attempt receives an actionable diagnostic; bus message published |
| Kill one session with `kill -9` | Reaper releases its leases within 90s; `agent.retired` audit row written |
| Heartbeat interrupted (network blackout) | Watcher flips agent state to `unreachable` after 90s; recovers when network returns |
| Convergio daemon stopped | Skill hooks surface a warning; do not block user work; reconnect resumes registration |

## What this PRD explicitly does *not* deliver

- **Runner adapter for headless agent execution.** This PRD wires
  Claude Code as an interactive session client. Headless runners
  (`spawn_runner` for autonomous agents) is a separate PRD,
  Wave 2.
- **Claude Agent SDK adapter.** SDK-driven agents are also Wave 2.
- **Copilot, Cursor, Codex CLI adapters.** Same skill pattern,
  separate PRDs, Wave 2 scoped per vendor.
- **Conflict resolution UX.** When two sessions try to lease the
  same file, this PRD surfaces the conflict; it does not
  implement a merge UI. That is post-Wave-3 work.
- **Multi-host coordination.** Single-machine, single-daemon only.
  Cross-machine remains explicitly out of scope per CONSTITUTION.

## Risks

- **Hook latency.** Adding an HTTP call before every Edit/Write
  could measurably slow the agent. Mitigation: lease claims are
  fire-and-forget on the happy path (response is 200 in single
  digit ms locally); only conflicts block. Watch the latency in
  telemetry once shipped.
- **`.claude/settings.json` proliferation.** Each repo adopting
  Convergio needs the same wiring. Mitigation: extend the
  existing `cvg setup agent claude` installer (commit `85332ea`,
  which today ships `mcp.json` + `prompt.txt`) to *also* write a
  `.claude/settings.json` hook template alongside the MCP files.
  Wave 0b task w1.5 is therefore "extend", not "create".
- **Hook reliability across Claude versions.** Claude Code hook
  semantics evolve (we have seen new hooks added in 2026 Q2).
  Mitigation: pin against the documented hook surface; the
  installer warns on Claude versions older than the supported
  baseline.

## Estimated effort

Revised after Artefact 4 was added (the original 9-13 day figure
covered Artefacts 1-3 only). Schema migration for system topics
and `cvg status --agents` plumbing are non-trivial; the pre-stop
check is the structural antibody for the day-end failure mode and
deserves its own slice.

- 2-3 days — skill + hook wiring + initial `/cvg-attach`,
  including correct endpoint use
  (`/v1/agent-registry/agents`)
- 1-2 days — small ADR + bus schema migration to allow
  `plan_id IS NULL` for `system.session-events` topic
- 2-3 days — `cvg status --agents` flag + JSON/plain output
  + i18n EN/IT + E2E test
- 2-3 days — `cvg session pre-stop` (Artefact 4): 6 checks +
  Stop-hook integration + audit row for forced-quit scenarios
  + E2E tests for each check
- 2 days — telemetry, lease-conflict diagnostic surfacing,
  reaper integration
- 1 day — extend `cvg setup agent claude` installer to write the
  `.claude/settings.json` hook template (the MCP-side scaffolding
  already shipped in `85332ea`)
- 1-2 days — README + dogfood demo (two sessions visible end
  to end, *including* a deliberate pre-stop refusal) + audit
  chain verification of the demo

**Total: ~12-16 days of focused work** (≈ 3 calendar weeks for
a single developer with normal context-switching). Lands as
its own PR, separate from the Wave 0a docs PR (see ROADMAP
Wave 0 split).
