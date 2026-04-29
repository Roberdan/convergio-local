# Convergio

> **The local operating system for safe parallel AI agents.**

Convergio runs on your machine and coordinates agent work before it can
be trusted. The current local runtime gives agents durable tasks,
evidence, audit, MCP integration, and server-side gates. The next
foundation adds CRDT-aware state, resource leases, patch proposals, and
merge arbitration before the first public release.

It is not an agent framework and it is not a cloud service. Bring your
own agent runner. Convergio gives that runner a local source of truth and
a mergeable coordination layer so multiple agents can work in parallel
without silently corrupting state, Git, or the filesystem.

## Why Convergio

The hard failure mode for coding agents is not one bad completion. It is
many agents working at once:

- overwriting the same files;
- diverging across worktrees;
- producing broken merges and noisy CI;
- losing process-local state;
- claiming "done" without evidence.

Convergio's design answer is:

1. durable task/evidence state;
2. hash-chained audit;
3. CRDT-aware multi-actor metadata;
4. workspace resource leases;
5. patch proposals and merge arbitration;
6. server-side gates that refuse unsafe `submitted`/`done` transitions.

Items 1, 2, and 6 are implemented in the local runtime today. Items 3,
4, and 5 are the next foundation before a public v0.1 release.

See [docs/vision.md](./docs/vision.md) for the product vision.

Repository naming: this public repo is intended to be
`convergio-local`, while the product and installed binaries remain
`Convergio`, `convergio`, `cvg`, and `convergio-mcp`.

## Principles enforced as code

1. **Zero tolerance for technical debt, errors and warnings.**
   `NoDebtGate`, `ZeroWarningsGate` and `NoSecretsGate` refuse
   `submitted`/`done` transitions when evidence contains debt markers,
   non-clean build/lint/test signals, or common credential leaks.
2. **Security first, local first.** The daemon binds to localhost by
   default, stores data in a local SQLite file, and treats evidence as
   untrusted input.
3. **Accessibility first.** CLI output must remain screen-reader
   friendly and must not rely on color alone.
4. **No scaffolding only.** `NoStubGate` refuses work that admits it is
   a stub, placeholder, skeleton, or not wired.
5. **Internationalization first.** CLI user-facing strings go through
   Fluent bundles with English and Italian shipped together.

See [CONSTITUTION.md](./CONSTITUTION.md) for the full rule set.

## Quickstart

```bash
sh scripts/install-local.sh
cvg setup

convergio start
```

In another terminal:

```bash
cvg doctor
cvg health
cvg demo
```

Optional daemon service:

```bash
cvg service install
cvg service start
```

For agents:

```bash
cvg setup agent claude   # or cursor, cline, continue, qwen, shell, copilot-local
cvg mcp tail             # inspect bridge diagnostics
```

Agent hosts that support MCP should connect the stdio command
`convergio-mcp`; it exposes only `convergio.help` and `convergio.act`.
See `docs/agents/README.md` for host-specific setup snippets.

Release artifacts can be built locally with `scripts/package-local.sh`
and signed/notarized on macOS with `scripts/sign-macos-local.sh`; see
`docs/release.md`.

Defaults:

- SQLite database: `~/.convergio/v3/state.db`
- HTTP bind: `127.0.0.1:8420`
- No external services
- No account, tenant, or server setup

You can override the local database file when needed:

```bash
convergio start --db sqlite:///tmp/convergio.db?mode=rwc
```

## Manual local loop

```bash
cvg plan create "ship one clean task" --project convergio-local
cvg status
cvg task list <plan_id>
cvg task transition <task_id> in-progress --agent-id local-agent
cvg evidence add <task_id> --kind code --payload '{"diff":"fn main() {}"}' --exit-code 0
cvg evidence add <task_id> --kind test --payload '{"warnings_count":0,"errors_count":0,"failures":[]}' --exit-code 0
cvg task transition <task_id> submitted --agent-id local-agent
cvg task transition <task_id> done --agent-id local-agent
cvg validate <plan_id>
cvg audit verify
```

Use `cvg demo` first: it creates one dirty task that gets refused by the
gates, then one clean plan that validates and verifies the audit chain.

## What you get

| Layer | Crate | What it gives you |
|-------|-------|-------------------|
| 1. Durability Core | `convergio-durability` | Plans, tasks, evidence, hash-chained audit log, gate pipeline, reaper loop |
| 2. Agent Message Bus | `convergio-bus` | Persistent topic/direct messages with ack, scoped per plan |
| 3. Agent Lifecycle | `convergio-lifecycle` | Spawn, heartbeat, process status, watcher loop |
| 4. Reference CLI flow | `convergio-planner`, `convergio-executor`, `convergio-thor`, `convergio-cli` | Minimal solve, dispatch and validate loop on top of layers 1-3 |

Layer 4 is intentionally small. The product value is the local runtime
and its gates; your own agent client can call the HTTP API directly.

## Project status

**v0.1 - local-first SQLite MVP.**

Current scope:

- SQLite-only local runtime
- localhost HTTP API
- `cvg status` dashboard for active plans and recently completed work
- hash-chained audit verification
- server-side quality gates
- common local secret-leak refusal
- persistent local message bus
- process spawn/heartbeat/watcher
- deterministic reference planner, executor tick, Thor validator and
  guided demo
- English/Italian CLI messages for the localized surfaces

Out of scope for this MVP:

- remote multi-user deployment
- account, tenant, or RBAC model
- graphical UI
- hosted service
- agent marketplace

The workspace has **171 tests** covering the local runtime, gates, audit
tamper detection, CLI smoke behavior, and HTTP E2E workflows.

## Documentation

- [ARCHITECTURE.md](./ARCHITECTURE.md) - layers, API and request lifecycle
- [CONSTITUTION.md](./CONSTITUTION.md) - non-negotiable rules
- [docs/vision.md](./docs/vision.md) - product vision and positioning
- [docs/multi-agent-operating-model.md](./docs/multi-agent-operating-model.md) - how multiple agents coordinate through one daemon
- [ROADMAP.md](./ROADMAP.md) - focused local-first roadmap
- [CONTRIBUTING.md](./CONTRIBUTING.md) - development workflow
- [docs/adr/](./docs/adr/) - architecture decision records

## License

Convergio Community License v1.3 (source-available, not OSI-approved). See [LICENSE](./LICENSE).
