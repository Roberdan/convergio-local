# Convergio Constitution

These rules are non-negotiable. They exist to keep us from drifting into
a generic "agent platform" and to keep the daemon honest.

The first three rules are **product principles**. Everything else
serves them.

---

# Sacred principles

## P1. Zero tolerance for technical debt, errors and warnings

In any language, in any output an agent attaches as evidence of work
done. No `TODO`, no `FIXME`, no `unwrap()`, no `console.log`, no
`pdb.set_trace`, no ignored tests, no `as any`, no `// nolint`, no
`fatalError`, no debug prints. Build must be clean. Tests must pass.
Linters must be silent.

The agent has two options: produce work that meets the bar, or get
its transition refused at the gate. There is no third option called
"merge it for now and clean up later".

Operationally: `NoDebtGate` and `ZeroWarningsGate` (Layer 1) refuse
`submitted`/`done` transitions when evidence carries debt markers or
non-clean quality signals. New languages and tools land as additional
rules in those gates. See ADR-0004.

## P2. Security first — including LLM security

Security is not a checklist for late milestones. It is a precondition.

This means:
- HMAC auth on every request in team mode (no shared secrets in code,
  no plaintext tokens)
- No secrets ever in evidence, code, logs (gate refuses on detected
  AWS keys, GitHub tokens, JWTs, ssh keys, `.env` blocks)
- Dependency audit (`cargo audit`, `npm audit`, `pip-audit`) is part
  of the evidence the agent must attach for any task that touches
  dependencies
- LLM-specific: prompt-injection patterns refused at the gate
  (`Ignore previous instructions`, role-confusion, system-prompt
  leak), no PII in evidence payloads, no model output trusted
  blindly when it claims authority

Operationally: future `NoSecretsGate`, `DepsAuditGate`,
`PromptInjectionGate`. Until they ship, the principle still applies
and code review is the safety net.

## P3. Accessibility first

Accessibility is a principle, not a polish step.

Two angles:

1. **The agent's output must be accessible.** When the agent produces
   UI code (HTML/JSX/SwiftUI/etc.), the gate refuses on `<img>` without
   alt, `<button>` rendered as `<div>`, color-only-information,
   placeholder-as-label, missing ARIA, contrast violations.
2. **Convergio itself is accessible.** The CLI offers `--format` modes
   (`human`, `json`, `plain`) with no ANSI escape codes when the
   terminal does not support them. Error messages are structured for
   screen readers. No information conveyed by color alone.

Operationally: future `A11yGate` for output, CLI a11y review in
sessione 8. The principle applies regardless: any feature that breaks
accessibility is a bug, not a trade-off.

## P4. No scaffolding only — every feature must be fully wired

The agent's most viscerally hated failure mode: declare something
"done" while leaving it disconnected, half-written, or invisible to
the rest of the codebase.

Three sub-failures, all unacceptable:

1. **Scaffolding only** — the agent creates `routes/foo.rs` but never
   adds `.merge(routes::foo::router())` to the app. The file exists,
   the feature does not.
2. **Disconnected feature** — the agent adds `pub fn bar()` but no
   caller exists. The function exists; dead code lives.
3. **Lying / forgetting** — the agent claims "I added the tests" or
   "I wired this in lib.rs" while the diff contains neither.

Operationally:

- `NoStubGate` refuses evidence whose payload contains explicit
  scaffolding markers: `// stub`, `// scaffolding`, `// placeholder`,
  `// to be wired`, `// not yet wired`, `// not connected`,
  `// (skeleton)`, `unreachable!()` (when used as "I'll get to it"
  rather than for genuine unreachable code).
- (Planned) `WireCheckGate` parses the diff: for each new module
  declared, ensures it is imported by a parent; for each new
  `pub fn`, ensures at least one caller exists in the diff or in
  existing code; for each new file under `routes/`, ensures it is
  merged into `app.rs`.
- (Planned) `ClaimCheckGate` requires evidence of kind `wire_check`
  with structured claims (`{type: "test_added", name: "test_foo"}`,
  etc.) and verifies each claim against the diff before allowing
  `submitted`.

The principle: **if the agent says "done", the work must actually be
reachable from `main` or from a test**. No exceptions.

---

These four are **non-negotiable**. They are not "nice to have", they
are not "v2 features", they are not "for enterprise customers". They
are **what Convergio is**. Removing any of them removes the product.

The technical rules below all serve P1, P2, P3, P4. When in doubt, the
principles win.

---

# Technical non-negotiables

## 1. Same binary, two modes

There is **one** `convergio` binary. The mode (personal vs team) is a
function of `CONVERGIO_DB`:

- `sqlite://...` (or unset → defaults to `~/.convergio/state.db`) → personal
- `postgres://...` → team

A new mode is **not** a new binary or a new fork. It is a config branch
in three known places (DB pool init, migration selection, auth middleware).

## 2. Cooperate, don't compete

LangGraph, CrewAI, Claude Code skills, AutoGen, Mastra: these are clients,
not competitors. We give them durability, audit and supervision. We do not
ship a DSL, a chain abstraction, or a "Convergio agent framework".

## 3. Reference implementation is part of the product

Layer 4 (`planner`, `thor`, `executor`) ships in the same repo as the
durability layer. It exists so a new user can `convergio start` and see
something useful in 5 minutes. It is not the product — but without it
the product is unsellable.

## 4. Anti-feature creep

These are deferred or cut, period:

- Mesh / multi-host (deferred — until a customer asks)
- Knowledge / catalog / org model (cut — plan + task + evidence is the model)
- Billing (cut — OSS only for now)
- Kernel / MLX (deferred — model agnostic)
- Night agents (deferred — Layer 3 + cron is enough)
- Skills marketplace (cut — never)
- 130+ MCP tools (reduced to ~15 covering layers 1-3 only)

If a feature is not in the 4 layers and not in the roadmap, it does not get
built. Issues are filed in `v3-backlog`.

## 5. Every feature must be tweetable

If explaining a feature requires a diagram, the feature is either not ready
or not the right feature. Ship the explanation first.

## 6. Server-enforced gates only

A task cannot be marked `done` from the client. The daemon verifies evidence
and transitions state. Clients propose, the daemon disposes.

The gate pipeline is fixed:

```
identity → plan_status → evidence → test → pr_commit → wave_sequence → validator
```

Any new gate must be justified, documented in an ADR, and ship with tests.

## 7. Audit log is append-only and hash-chained

Every state transition writes a row to `audit_log` whose `hash` is
`sha256(prev_hash || canonical_json(payload))`. The chain is verifiable
via `GET /v1/audit/verify` from any external process.

Mutating an audit row, or breaking the chain, is a bug, not a feature.

## 8. No SQLite-specific SQL leaks

Schema, migrations and queries must work on both SQLite and Postgres. Where
behavior differs, abstract behind `convergio-db`. CI runs the test suite
against both backends (Postgres added in week 1 of MVP).

## 9. CLI is a pure HTTP client

`cvg` MUST NOT import server crates. It speaks HTTP. A contract test
enforces this.

## 10. Loop must close

Every feature has: input → processing → output → feedback → state update →
visible to the user. If the user can't see the result, it is not done.

## 11. Tests are the spec

If behavior is not under test, it is not guaranteed. Public APIs
(HTTP routes and library `pub fn`) require tests. Bug fixes require a
regression test.
