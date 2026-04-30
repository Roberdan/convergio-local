# Multi-agent operating model

This document answers the practical question: how do multiple Claude,
Copilot, Cursor, Cline, shell, or custom agents use one Convergio without
creating chaos?

## Short version

Agents do not coordinate by chatting directly. They coordinate by using
the same local Convergio daemon.

```text
Claude Code  ─┐
Copilot CLI  ─┤
Cursor agent ─┼──> convergio-mcp / HTTP / cvg ──> Convergio daemon
Cline        ─┤                                  └─> SQLite + audit + gates
shell agent  ─┘
```

Convergio is the shared state, lock manager, message bus, evidence
store, gatekeeper, and future merge arbiter. Agents are workers.

## Two valid ways to use it

### Mode 1: human-opened swarm

The user opens multiple agent sessions manually:

```text
Terminal 1: Claude Code
Terminal 2: Copilot CLI
Terminal 3: Cursor agent
Terminal 4: shell runner
```

Each host is configured with the same MCP bridge:

```bash
cvg setup agent claude
cvg setup agent copilot-local
cvg setup agent cursor
```

All of them point to:

```text
http://127.0.0.1:8420
```

Each agent calls `convergio.help`, gets the same protocol, asks for work
with `next_task`, claims one task, heartbeats, adds evidence, submits,
and obeys refusals.

This is the first practical multi-agent mode.

### Mode 2: Convergio-orchestrated swarm

A lead agent or human creates a plan. Convergio decomposes or receives
tasks, then launches worker agents through registered runner adapters.

```text
user/lead agent
  -> create plan
  -> solve plan into tasks
  -> dispatch runnable tasks
  -> runner adapters spawn workers
  -> workers claim/heartbeat/evidence/submit
```

This requires real runner adapters for Claude, Copilot, shell, or other
tools. The lifecycle crate can supervise processes, but product-quality
runner adapters are future work. Until those adapters exist, use Mode 1.

## What a single agent must do

Every agent session needs a unique `agent_id`, for example:

```text
claude-architect-01
copilot-impl-03
cursor-reviewer-02
```

The loop is:

1. Call `convergio.help`.
2. Call `agent_prompt` to get the current Convergio instructions.
3. Call `status`.
4. Use the active-plan dashboard to understand current work.
5. Get work with `next_task` or receive an assigned task.
6. Claim it with `claim_task`.
7. Send heartbeat while working.
8. Fetch task context with `get_task_context`.
9. Coordinate through `poll_messages`, `publish_message`, and
   `ack_message`.
10. For workspace-changing tasks, request leases for
    files/directories/symbols.
11. Work in an isolated sandbox/worktree.
12. Submit a patch proposal instead of merging directly.
13. Wait for the merge arbiter and gates.
14. Add evidence.
15. Submit.
16. If refused, read `explain_last_refusal`, fix, add new evidence, retry.
17. Only report done after Convergio accepts.

## Does the database act as context?

Yes, but not as a giant chat transcript.

The database is durable operational context:

| Context type | Stored in Convergio |
|--------------|---------------------|
| plan goal | plan record |
| task scope | task record |
| dependencies | task graph |
| agent identity | agent/session record |
| instructions | agent prompt + task description |
| progress | heartbeat + task status |
| discussion | message bus |
| facts/proof | evidence |
| refusal reasons | audit + gate output |
| future conflicts | CRDT/workspace conflict records |

Agents should not paste entire conversations into every task. Convergio
should give each worker a compact task packet:

```text
plan summary
task objective
constraints
allowed resources
relevant prior evidence/messages
required output/evidence
local folder instructions
```

That is how we avoid boiling agents with too much context.

## Should agents talk to each other?

Not directly.

Direct agent-to-agent chat is invisible, unaudited, and impossible to
replay. Agents may communicate through Convergio:

| Need | Channel |
|------|---------|
| announce progress | task status / heartbeat |
| ask another role for input | message bus topic |
| hand off findings | evidence |
| block unsafe work | lease/conflict/refusal |
| explain failure | audit/refusal record |

The message bus is the communication channel. It is persisted in SQLite,
scoped to a plan, and can be replayed. Agents can have skills/roles, but
coordination still goes through the daemon.

## Agent names, roles, and skills

Convergio needs three separate concepts:

| Concept | Example | Purpose |
|---------|---------|---------|
| `agent_id` | `claude-impl-01` | unique running worker identity |
| `actor_id` | UUID | CRDT identity for writes/imported ops |
| role/skills | `rust`, `review`, `docs` | scheduling and task matching |

Do not overload one field for all three.

## What works today

Implemented today:

- one local daemon;
- one SQLite state file;
- MCP bridge with `convergio.help` and `convergio.act`;
- plan/task/evidence lifecycle;
- task claim and heartbeat;
- gate refusals;
- durable refusal explanation;
- hash-chained audit;
- local service management;
- host setup snippets;
- durable agent registry;
- task context packet generator;
- plan-scoped bus actions through `convergio.act`;
- CRDT storage/import/conflict foundation;
- workspace resources, leases, patch proposals, conflicts, and merge queue;
- local capability registry and signature verification.

Partially available today:

- process lifecycle/supervision exists, but real Claude/Copilot runner
  adapters are not productized.

Not implemented yet:

- skill-aware scheduling;
- local signed capability install/rollback;
- downloaded capability runners;
- planner as an installed capability.

## What must be built next

To make this feel like "open one Convergio plan and let it run a swarm",
the next core pieces are:

1. **Runner adapters** — Convergio can spawn known agent runners when the
   user wants orchestration instead of manual swarm sessions.
2. **Signed capability install/rollback** — capabilities can be installed
   locally only after signature verification.
3. **Planner capability** — planner behavior moves behind the capability
   action boundary instead of growing the core.

## Anti-chaos rules

1. Agents never write directly to Convergio SQLite.
2. Agents never coordinate important decisions outside Convergio.
3. Agents never mark work complete without accepted Convergio state.
4. Agents never mutate the canonical workspace directly once leases and
   patch proposals exist.
5. Context is task-scoped, not repo-wide chat history.
6. Every crate/folder has local agent instructions for responsibility and
   invariants.
7. New orchestration behavior lives behind daemon APIs and tests, not
   only in prompts.

## The mental model

Convergio is not "a better prompt".

Convergio is the local control plane:

```text
planner creates tasks
workers claim tasks
leases protect resources
evidence proves work
gates refuse unsafe transitions
messages coordinate handoffs
patch proposals protect Git
merge arbiter updates canonical state
audit proves what happened
```

The agents can be Claude, Copilot, Cursor, Cline, shell scripts, or
future capabilities. The rule is the same: they work through Convergio,
not around it.
