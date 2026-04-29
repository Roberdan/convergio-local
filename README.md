# Convergio

> **A local SQLite-backed runtime that stops AI agents from claiming
> "done" without evidence.**

Convergio runs on your machine, watches agent work through a local HTTP
daemon, and refuses task transitions when the attached evidence violates
the product rules: technical debt, warnings, missing evidence, or
scaffolding-only work.

It is not an agent framework and it is not a cloud service. Bring your
own agent runner (Claude Code, LangGraph, CrewAI, shell scripts, custom
Python). Convergio gives that runner local durability, an audit trail,
agent messaging, process supervision, and server-side gates.

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
cvg plan create "ship one clean task"
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
- [ROADMAP.md](./ROADMAP.md) - focused local-first roadmap
- [CONTRIBUTING.md](./CONTRIBUTING.md) - development workflow
- [docs/adr/](./docs/adr/) - architecture decision records

## License

Apache 2.0. See [LICENSE](./LICENSE).
