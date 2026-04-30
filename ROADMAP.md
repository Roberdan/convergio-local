# Roadmap

Focused MVP: **single-user, local-first, SQLite-only Convergio**.

The goal is not to become a hosted platform. The goal is to solve
one concrete problem well: **local AI agents should not be able to
claim "done" before evidence, gates and audit accept the work**.

For the day-to-day status see [`STATUS.md`](./STATUS.md).

## Shipped — v0.1.x

The runtime that v0.1.0 advertised, plus everything the
office-hours dogfood session proved was missing.

- [x] SQLite-backed daemon, localhost HTTP API, `cvg` CLI
- [x] Plans / tasks / evidence / audit_log
- [x] Hash-chained audit verification (`GET /v1/audit/verify`,
      ADR-0002)
- [x] Gate pipeline: PlanStatus, Evidence, NoDebt (7 languages),
      NoStub, ZeroWarnings, NoSecrets, WaveSequence
- [x] Persistent plan-scoped agent message bus (Layer 2)
- [x] Agent process spawn / heartbeat / watcher (Layer 3)
- [x] Reaper for stale `in_progress` tasks
- [x] Reference planner + Thor + executor tick (Layer 4)
- [x] CRDT actor/op store, workspace leases, patch proposals,
      merge arbiter (ADR-0006, ADR-0007)
- [x] Local capability registry + Ed25519 signed `install-file`
      (ADR-0008)
- [x] Local shell runner adapter proof
- [x] Setup + doctor + service + MCP bridge + adapter snippets
- [x] English / Italian i18n with coverage gate
- [x] PR template with `## Files touched` manifest
- [x] CONSTITUTION § 13 agent context budget + CI audit
- [x] CONSTITUTION § 15 worktree discipline
- [x] CONSTITUTION § 16 legibility score + CI audit
- [x] `cvg pr stack` (PR queue dashboard, read-only)
- [x] `cvg task create` with rich fields
- [x] `cvg plan create / list / get` + `cvg task list` honor
      `--output human|json|plain`
- [x] `examples/claude-skill-quickstart/` — runnable before/after
      demo (no LLM required)
- [x] Friction-log discipline (`docs/plans/v0.1.x-friction-log.md`)
- [x] Repo public on `Roberdan/convergio-local`, release-please
      workflow producing v0.1.x tags

## Shipped or in v0.2.0 (release-please PR #18)

- [x] **ADR-0011**: `done` is set only by Thor; agents may submit
      but only `cvg validate` promotes (BREAKING change documented
      via `Action::CompleteTask` removal + `SCHEMA_VERSION` bump
      from `1` to `2`)
- [x] **Wave-sequence gate fix**: `failed` is terminal, does not
      block subsequent waves
- [x] **ADR-0010**: retire the empty `convergio-worktree` crate
- [x] **ADR-0012**: OODA-aware validation — the spine for the
      smart-Thor / negotiation / escalation / multi-vendor work
      below

## Next: v0.2.x finishing wave

Small, scoped, immediate.

- [ ] **T2.04** auto-close plan task on PR merge
      (`cvg pr sync <plan>` parses merged PRs for `Tracks` lines
      and transitions the linked task)
- [ ] `cvg pr stack` localised to EN/IT and validating the
      manifest against the real diff
      (paused branch `fix/cvg-pr-stack-i18n-and-manifest-validation`)
- [ ] **T1.17** Tier-2 retrieval: machine-readable YAML
      frontmatter on every ADR + `cvg coherence check` for cross-
      references
- [ ] **T3.08** trim the 12 Rust files in 250-300 LOC range to
      leave headroom

## v0.3 — smart Thor + outcome reliability (ADR-0012)

The validator stops being an evidence-shape check and starts
verifying outcomes. Karpathy's 2026 LLM-Wiki idea fuses with
Convergio's audit chain here.

- [ ] **T3.02** smart Thor — `cvg validate` runs the project's
      actual pipeline (cargo fmt / clippy / test / doc-check / ADR
      coherence) before Pass
- [ ] **T3.03** agent ↔ Thor negotiation via `propose_plan_amendment`
- [ ] **T3.04** 3-strike escalation to the human operator
- [ ] **T3.06** wave-scoped validation (`cvg validate <plan>
      --wave N` — waves treated as PRs)
- [ ] **T3.07** wave-aware planner that bundles tasks into
      coherent ship units
- [ ] **T2.05** split `convergio-durability` (8059 LOC today) into
      audit+gates / plan-task-evidence / workspace+crdt+capability
      sub-crates so each fits an agent's context window
- [ ] **T4.01** context packet: Thor receives project rules + past
      refusals + crate AGENTS.md + related ADRs as input
- [ ] **T4.02** learnings store: query view over the audit chain
      so refusals on the same pattern are surfaced to the agent
- [ ] **T4.03** agent reputation aggregated from audit history
- [ ] **T4.05** durable agent sessions for long runs (rehydrate
      across host restarts)

## v0.4+ — multi-vendor + AI-maintained knowledge

The OODA loop becomes a swarm. Multiple agents from multiple
vendors collaborate; the wiki grows under their hands.

- [ ] **T4.04** multi-vendor model routing (Codex CLI, Copilot,
      Cursor, Continue, Cline) as registered agents with
      cost / latency / capability profiles
- [ ] **T4.06** fresh-eyes legibility simulation (zero-shot agent
      comprehension test)
- [ ] **T4.07** local RAG over the corpus (SQLite FTS5 + optional
      embeddings)
- [ ] **T4.08** **LLM Wiki**: `docs/learnings/<topic>.md`
      AI-maintained knowledge base, fed by Thor refusals and
      compacted by a periodic agent
- [ ] **T3.09** Tolaria-style frontmatter compatibility +
      `cvg setup vault` to bridge a Tolaria vault into the
      `convergio.help` surface

## Explicitly out of scope

- hosted service
- remote multi-user deployment
- account / organisation model
- RBAC
- distributed mesh
- graphical UI
- billing
- agent marketplace

## Success criteria

- A solo developer can install the daemon and CLI, run the
  quickstart, and see a gate refusal plus audit verification in
  minutes.
- A multi-agent team can work in parallel without directly
  mutating the canonical workspace or silently overwriting each
  other's state.
- The legibility score (CONSTITUTION § 16) stays above 70 / 100;
  any drop is investigated as a regression.
- Every refusal Thor produces is non-falsifiable and fixable —
  the agent receives a structured pointer, not a blank "no".
