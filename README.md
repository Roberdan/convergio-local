# Convergio

[![CI](https://github.com/Roberdan/convergio-local/actions/workflows/ci.yml/badge.svg)](https://github.com/Roberdan/convergio-local/actions/workflows/ci.yml)
[![Release](https://github.com/Roberdan/convergio-local/actions/workflows/release.yml/badge.svg)](https://github.com/Roberdan/convergio-local/actions/workflows/release.yml)
[![License: Convergio Community](https://img.shields.io/badge/license-Convergio%20Community-blue)](https://github.com/Roberdan/convergio-local/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange)](https://www.rust-lang.org/)
[![Zero Warnings](https://img.shields.io/badge/warnings-0-brightgreen)](#)

> **A local daemon that refuses AI-agent work whose evidence does not
> match the claim of done — and writes every refusal to a hash-chained
> audit log.**

Convergio runs on your machine, sits between your agent runner and
your codebase, and applies server-side gates to every `submitted` /
`done` transition. When the evidence the agent attaches contains
debt markers, scaffolding tells, non-clean build signals, or
credential leaks, Convergio returns 409 and records the refusal in
an audit chain you can verify from outside.

It is not an agent framework and it is not a cloud service. Bring
your own agent runner (Claude Code, a Python loop, a shell script).
Convergio gives that runner a durable local source of truth, a gate
pipeline, and a mergeable coordination layer so multiple agents can
work in parallel without silently corrupting state, Git, or the
filesystem.

The honest mechanism, in one line: Convergio cannot make an agent
truthful, but it raises the cost of lying and makes every refusal
non-falsifiable.

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

All six are implemented in the local runtime. `v0.1.0` also includes
signed local capability install/remove, a `planner.solve` capability
action, and a constrained local shell runner proof. Product-quality
runner adapters beyond the shell proof, remote capability registry and
ACP bridge remain roadmap work.

See [docs/vision.md](./docs/vision.md) for the product vision.

Repository naming: this public repo is intended to be
`convergio-local`, while the product and installed binaries remain
`Convergio`, `convergio`, `cvg`, and `convergio-mcp`.

## Principles, and which ones are actually enforced today

The five principles below are the product's identity. Each one carries
an explicit status — `enforced`, `partial`, or `planned` — so the
README does not claim more than the code does.

1. **P1 — Zero tolerance for technical debt, errors and warnings.**
   `enforced`. `NoDebtGate` (7 languages), `NoStubGate`, and
   `ZeroWarningsGate` refuse `submitted`/`done` transitions when
   evidence contains debt markers, scaffolding tells, or non-clean
   build/lint/test signals.
2. **P2 — Security first, local first.** `partial`. Localhost-by-default
   bind, evidence-as-untrusted-input, and `NoSecretsGate` (gitleaks
   pattern set) are shipped. `DepsAuditGate`, `PromptInjectionGate`,
   and HMAC middleware for non-loopback bind remain roadmap.
3. **P3 — Accessibility first.** `planned`. No `A11yGate` yet. CLI
   strives to remain screen-reader friendly without color, but this
   is convention rather than enforcement until the gate ships.
4. **P4 — No scaffolding only.** `enforced` for self-admitted stubs.
   `NoStubGate` refuses evidence that says it is a stub, placeholder,
   skeleton, or not wired. `WireCheckGate` (proves new symbols have
   real callers in the diff) remains roadmap.
5. **P5 — Internationalization first.** `enforced`. CLI user-facing
   strings go through Fluent bundles with English and Italian
   shipped together; a coverage test refuses partial locales.

See [CONSTITUTION.md](./CONSTITUTION.md) for the full rule set, and
[docs/plans/v0.1.x-friction-log.md](./docs/plans/v0.1.x-friction-log.md)
for the gaps the next release will close.

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
- task context packets and plan-scoped bus actions for MCP agents
- CRDT actor/op schema, deterministic import/merge and conflict surfacing
- workspace leases, patch proposals and merge queue arbitration
- process spawn/heartbeat/watcher and local shell runner proof
- local capability registry, Ed25519 package signature verification, and
  signed local `install-file`/remove
- `planner.solve` as the first installed capability-gated action
- deterministic reference planner, executor tick, Thor validator and
  guided demo
- English/Italian CLI messages for the localized surfaces

Out of scope for this MVP:

- remote multi-user deployment
- account, tenant, or RBAC model
- graphical UI
- hosted service
- agent marketplace

The workspace test suite covers the local runtime, gates, audit tamper
detection, CLI smoke behavior, CRDT/workspace flows, MCP actions, and HTTP
E2E workflows.

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
