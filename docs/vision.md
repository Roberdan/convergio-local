# Convergio vision

AI agents are easy to start and hard to trust. The hard failure mode is
not one agent making one mistake; it is many agents working in parallel,
overwriting each other's state, fighting over files, creating broken Git
merges, triggering noisy CI, and still claiming "done".

Convergio is being built as the local coordination layer for that world.

The current local runtime already provides durable tasks, evidence,
audit, gates, MCP, service management, and release packaging. The next
foundation adds CRDT-aware state, resource leases, patch proposals, and a
merge arbiter before the first public release.

## Product sentence

Convergio is the local operating system for safe parallel AI agents.

## What problem it solves

Local AI agents today usually share unsafe primitives:

| Primitive | Failure mode |
|-----------|--------------|
| Filesystem | agents overwrite the same files or generated artifacts |
| Git worktrees | parallel branches diverge and become expensive to reconcile |
| Pull requests | agents create review/CI noise faster than humans can triage |
| CI | every agent believes its local state is the truth |
| Task lists | "done" is claimed without evidence or auditability |
| Agent memory | state is lost when the process dies |

Convergio makes the coordination explicit. Agents can work in parallel,
but Convergio owns the durable state, gates, leases, audit log, and merge
queue.

## Design commitments

1. **Evidence before done.** A task cannot be submitted or completed
   without required evidence.
2. **Audit before trust.** State changes are hash-chained and verifiable.
3. **CRDT-aware state before sync.** Multi-actor state is modeled from day
   zero, even when all actors run on one machine.
4. **Leases before file edits.** Agents must claim the resources they
   intend to change.
5. **Patch proposals before merges.** Agents propose changes; Convergio
   arbitrates application to the canonical workspace.
6. **Capabilities before monolith.** New functionality is installed as
   signed, isolated capabilities, not as unbounded code inside the core.
7. **Stable agent protocol.** Agents use `convergio.help` and
   `convergio.act`; Convergio keeps the daemon as source of truth.

## Product shape

The public repository can be named `convergio-local` to distinguish it
from legacy experiments, but the product remains Convergio:

| Surface | Name |
|---------|------|
| Daemon | `convergio` |
| CLI | `cvg` |
| MCP bridge | `convergio-mcp` |
| Possible future ACP bridge | `convergio-acp` |
| Capability binaries | `convergio-cap-<name>` |

The first public product is local-first and single-user. That does not
mean single-agent. The local core must already support many agents on one
machine and must not block future multi-machine synchronization.

## Extension model

Convergio core should stay small:

- SQLite local state
- CRDT operation log and materialized state
- tasks, evidence, gates, audit
- agent message bus and lifecycle
- workspace resource leases and merge queue
- MCP/CLI/HTTP interfaces
- capability manager

Future additional behavior is installed on demand:

```bash
cvg capability install planner
```

Capabilities are signed packages with a manifest, isolated process,
optional capability-local database, declared actions, declared doctor
checks, migrations, docs, and tests.

## Protocol positioning

| Protocol | Role |
|----------|------|
| HTTP | daemon API and source-of-truth boundary |
| CLI (`cvg`) | human/admin interface |
| MCP | tool interface for agents |
| ACP | future editor/IDE-facing agent-client interface |

MCP and ACP are complementary. MCP lets agents call Convergio as a tool.
ACP can let editors talk to Convergio as an agent/proxy. Neither may
bypass gates, evidence, or audit.

## What v0.1 must prove

Before the first public release, Convergio must prove:

- local install/setup/doctor works;
- MCP bridge works;
- audit/gates work;
- macOS package can be signed and notarized;
- CRDT storage foundation exists;
- two local actors can merge state deterministically;
- workspace leases and patch proposals prevent unsafe parallel edits;
- same-file/stale-base conflicts are surfaced instead of hidden;
- every accept/refuse/merge is audited.

If Convergio only tracks tasks but still lets agents corrupt Git and the
filesystem, it has not solved the real problem.
