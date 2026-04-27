# Convergio

> **The first runtime that imposes quality, security and accessibility
> on AI-agent output. Server-side. Before it ships.**

Agents lie. They leave technical debt without telling you. They skip
pieces of the plan. They claim "done" when they cut corners. Convergio
sits between your agent and your codebase and **refuses** the work
that does not meet the bar.

Five principles, enforced as code, not as slogans
(see [CONSTITUTION.md](./CONSTITUTION.md)):

1. **Zero tolerance for technical debt, errors, warnings** — in any
   programming language. No `TODO`, no `unwrap()`, no `console.log`,
   no `pdb.set_trace`, no ignored tests, no warnings, no failing
   builds. `NoDebtGate` + `ZeroWarningsGate` refuse `done`
   transitions when the evidence the agent itself attached contains
   debt markers or non-clean quality signals.
2. **Security first — including LLM security**. HMAC auth, no secrets
   in evidence, dependency audit gates, prompt-injection refusal.
3. **Accessibility first**. Output that violates a11y is not "polish
   for later", it is a refused transition. The CLI itself is
   designed for screen readers.
4. **No scaffolding only — every feature must be wired**. Files
   created without being imported, functions without callers,
   `// stub` and `# placeholder` comments — `NoStubGate` refuses.
5. **Internationalization first**. Italian and English first-class
   day one (`cvg --lang it health`); every user-facing string flows
   through Fluent bundles; partial-locale shipping is refused by
   the i18n coverage gate.

Underneath: a hash-chained audit log so nothing the agent did can be
silently rewritten, a reaper that survives agent death, and a single
binary that runs in personal (SQLite) or team (Postgres) mode.

It is **not** another agent orchestrator. It is the **leash** that
sits underneath whatever orchestrator (LangGraph, CrewAI, Claude
Code skills, your own Python) you already use.

---

## Quickstart (personal mode)

```bash
cargo install --path crates/convergio-cli   # until first crates.io release
convergio start                             # SQLite at ~/.convergio/state.db
cvg health                                  # ping the daemon
cvg plan create "my first plan"
cvg plan list
```

Defaults:

- `~/.convergio/state.db` (SQLite)
- `http://127.0.0.1:8420`
- localhost auth bypass (no HMAC required)

## Quickstart (team mode)

```bash
docker run -d \
  -e CONVERGIO_DB=postgres://user:pass@host/convergio \
  -e CONVERGIO_HMAC_KEY=$(openssl rand -hex 32) \
  -p 8420:8420 \
  ghcr.io/roberdan/convergio:latest
```

Differences from personal mode:

- HMAC signature required on every request
- Multi-tenant via `org_id`
- Postgres as durability + replication backbone
- Audit log hash chain verifiable by an external cron
  (`GET /v1/audit/verify`)

---

## What you get (4 layers)

| Layer | Crate | What it gives you |
|-------|-------|-------------------|
| 1. Durability Core | `convergio-durability` | Plans, tasks, evidence, hash-chained audit log, server-enforced gate pipeline, heartbeat reaper |
| 2. Agent Comm Bus | `convergio-bus` | Topic + direct message persistence with ack, scoped per plan |
| 3. Agent Lifecycle | `convergio-lifecycle` | Spawn, supervise, reap long-running agent processes |
| 4. Reference Impl | `convergio-planner` + `convergio-thor` + `convergio-executor` | Solver, validator, dispatcher built on top of layers 1-3 |

Layer 4 is a **reference** implementation. You are encouraged to delete it
and write your own client on top of layers 1-3 if your workflow needs something
different.

---

## Anti-goals

Things Convergio **explicitly does not do**:

1. We do **not** build a new agent framework. Bring your own
   (Claude Code, LangGraph, CrewAI, plain Python).
2. We do **not** compete with Temporal, LangGraph, MCP or A2A.
   They live above us, beside us, or solve a different problem.
3. We do **not** ship a UI in the MVP. CLI + JSON is enough.
4. We do **not** ship an agent marketplace, skill registry, or
   billing. Ever, until a paying customer asks.
5. We do **not** add "AI features" in the daemon
   ("AI suggests next task", etc). Layer 4 stays minimal.

See [CONSTITUTION.md](./CONSTITUTION.md) for the full list of non-negotiable
rules.

---

## Project status

**v0.1 — Layer 1 + 2 + 3 + 4 (basic) implemented.**

| Layer | Status |
|-------|--------|
| 1 — Durability core | done: plans, tasks, evidence, hash-chained audit, gate pipeline, reaper loop |
| 2 — Agent message bus | done: persistent publish/poll/ack with cursor, scope-per-plan |
| 3 — Agent lifecycle | done: spawn, heartbeat, mark_exited, OS-watcher loop |
| 4 — Reference impl | basic: deterministic planner, Thor validator, executor dispatch, CLI |

68 tests passing under fmt + clippy `-D warnings`. Audit chain
tamper-detection is proven by 6 dedicated tests
(`crates/convergio-durability/tests/audit_tamper.rs`). The full
`solve → dispatch → validate` quickstart is exercised end-to-end via
HTTP (`crates/convergio-server/tests/e2e_quickstart.rs`).

Personal mode SQLite only — Postgres team mode and HMAC auth are
deferred.

See [ROADMAP.md](./ROADMAP.md) for the plan and
[CHANGELOG.md](./CHANGELOG.md) for what shipped per session.

---

## Documentation

- [ARCHITECTURE.md](./ARCHITECTURE.md) — the 4 layers, crate boundaries, request lifecycle
- [CONTRIBUTING.md](./CONTRIBUTING.md) — how to develop, test conventions, commit style
- [CONSTITUTION.md](./CONSTITUTION.md) — non-negotiable rules
- [ROADMAP.md](./ROADMAP.md) — what is next
- [docs/adr/](./docs/adr/) — architecture decision records
- [docs/spec/](./docs/spec/) — original v3 specification

## License

Apache 2.0. See [LICENSE](./LICENSE).
