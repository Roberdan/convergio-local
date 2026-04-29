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
   `NoDebtGate` and `ZeroWarningsGate` refuse `submitted`/`done`
   transitions when evidence contains debt markers or non-clean
   build/lint/test signals.
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
cargo install --path crates/convergio-server
cargo install --path crates/convergio-cli

convergio start
cvg health
cvg plan create "my first plan"
cvg plan list
```

Defaults:

- SQLite database: `~/.convergio/state.db`
- HTTP bind: `127.0.0.1:8420`
- No external services
- No account, tenant, or server setup

You can override the local database file when needed:

```bash
convergio start --db sqlite:///tmp/convergio.db?mode=rwc
```

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
- persistent local message bus
- process spawn/heartbeat/watcher
- deterministic reference planner, executor tick and Thor validator
- English/Italian CLI messages for the localized surfaces

Out of scope for this MVP:

- remote multi-user deployment
- account, tenant, or RBAC model
- graphical UI
- hosted service
- agent marketplace

The workspace has **140 tests** covering the local runtime, gates, audit
tamper detection, CLI smoke behavior, and HTTP E2E workflows.

## Documentation

- [ARCHITECTURE.md](./ARCHITECTURE.md) - layers, API and request lifecycle
- [CONSTITUTION.md](./CONSTITUTION.md) - non-negotiable rules
- [ROADMAP.md](./ROADMAP.md) - focused local-first roadmap
- [CONTRIBUTING.md](./CONTRIBUTING.md) - development workflow
- [docs/adr/](./docs/adr/) - architecture decision records

## License

Apache 2.0. See [LICENSE](./LICENSE).
