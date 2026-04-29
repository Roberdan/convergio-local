# Convergio Constitution

These rules keep Convergio focused: a local runtime that refuses
low-quality AI-agent work before it is marked done.

---

# Sacred principles

## P1. Zero tolerance for technical debt, errors and warnings

In any language, in any output an agent attaches as evidence of work
done. No `TODO`, no `FIXME`, no `unwrap()`, no `console.log`, no
`pdb.set_trace`, no ignored tests, no `as any`, no `// nolint`, no
debug prints. Build must be clean. Tests must pass. Linters must be
silent.

Operationally: `NoDebtGate`, `ZeroWarningsGate` and `NoSecretsGate`
refuse `submitted`/`done` transitions when evidence carries debt
markers, non-clean quality signals, or common credential leaks.

## P2. Security first, local first

Convergio is a single-user localhost daemon. The safe default is:

- SQLite file on the user's machine
- HTTP bind on `127.0.0.1`
- no remote users, accounts, tenants, RBAC, or hosted control plane
- no secrets in evidence, logs, or code
- spawned agents run with the daemon user's privileges and are not a
  sandbox boundary

LLM-specific threats still matter. Prompt-injection patterns, secret
leaks, and suspicious evidence are treated as bugs in the gate surface,
not as "later" concerns. The MVP includes first-pass secret detection;
future security gates may add prompt-injection refusal, but the runtime
remains local-first.

## P3. Accessibility first

Accessibility is a principle, not a polish step.

1. Agent output that creates UI must be accessible.
2. Convergio's own CLI must be usable without color, animation, or
   terminal-specific assumptions.

Planned gates may scan UI evidence for common accessibility failures.
Until then, any feature that makes Convergio harder to use with assistive
technology is a bug.

## P4. No scaffolding only

If an agent says "done", the work must actually be reachable from code
or tests. Creating files without wiring them, leaving placeholders, or
shipping skeleton functions is not done.

Operationally: `NoStubGate` refuses `submitted`/`done` transitions when
evidence contains explicit scaffolding markers such as `stub`,
`placeholder`, `to be wired`, `not implemented`, `(skeleton)`, or
language-specific not-implemented constructs.

Planned deeper gates may parse diffs to prove new modules, routes, and
public functions are actually wired.

## P5. Internationalization first

The product must be usable by people who do not read English fluently.
Italian and English are first-class from day one.

Operationally:

- CLI user-facing strings flow through `convergio-i18n`
- Fluent bundles ship for `en` and `it`
- coverage tests assert both locales expose the same keys
- machine-readable API error codes stay stable English identifiers

---

# Technical non-negotiables

## 1. Local SQLite only

The runtime uses one local SQLite database file. No external database is
required or supported in the MVP. This is deliberate: zero setup, low
resource use, reproducible local state, and a small operational surface.

Default database:

```text
sqlite://$HOME/.convergio/v3/state.db?mode=rwc
```

## 2. Cooperate, don't compete

LangGraph, CrewAI, Claude Code skills, AutoGen, shell scripts and custom
Python are clients, not competitors. Convergio gives them local
durability, audit and gates. It does not provide a DSL, agent framework
or workflow language.

## 3. Reference implementation is part of the product

Layer 4 (`planner`, `thor`, `executor`) stays small but usable. It
exists so a new user can start the daemon and see a complete local loop
without writing a client first.

## 4. Anti-feature creep

Deferred or cut unless real local-user adoption proves the need:

- remote deployment
- account or organization model
- RBAC
- hosted service
- mesh / multi-host
- knowledge catalog
- billing
- skills marketplace

## 5. Every feature must close the loop

Every feature has: input -> processing -> output -> feedback -> state
update -> visible result. If the user cannot see the result, it is not
done.

## 6. Server-enforced gates only

A task cannot honestly be marked complete by the client alone. The
daemon verifies evidence and transitions state. Clients propose; the
daemon disposes.

The gate pipeline is fixed and must remain server-side:

```text
plan_status -> evidence -> no_debt -> no_stub -> zero_warnings -> wave_sequence
```

Any new gate must be documented and tested.

## 7. Audit log is append-only and hash-chained

Every audited state transition writes a row to `audit_log` whose `hash`
is `sha256(prev_hash || canonical_json(payload))`. The chain is
verifiable via `GET /v1/audit/verify`.

Mutating an audit row, or breaking the chain, is a bug.

## 8. CLI is a pure HTTP client

`cvg` must not import server crates. It speaks HTTP to the local daemon.

## 9. Tests are the spec

If behavior is not under test, it is not guaranteed. Public HTTP routes
and library APIs require tests. Bug fixes require regression tests.
