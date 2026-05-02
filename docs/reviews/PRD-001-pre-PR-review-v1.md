---
id: review-prd-001-pre-pr-v1
type: adversarial-review
target: docs/prd/0001-claude-code-adapter.md (entire Wave 0b PR)
template_version: v1
date: 2026-05-01
reviewer: claude-opus-4-7-1m (sibling-session second pass per ADR-0022)
language: English
---

# Adversarial review (pre-PR) — Wave 0b assembled

> Second adversarial pass for PR #62, Wave 0b. Reapplies the ADR-0022 v1 template to the assembled deliverable (updated PRD + ADR-0025 + bus migration + `/cvg-attach` skill + `/v1/system-messages` endpoint + `cvg setup agent claude` extension + `cvg agent list` flag + E2E test + demo). The v1 findings (`docs/reviews/PRD-001-adversarial-review-v1.md`) were addressed in commit `d68957c` and later commits; this pass looks for new problems introduced by the written code.

## A) Internal contradictions (top 5)

**A1 — w1.4b "cvg session pre-stop" is declared in PRD-001 § Artefact 4 but not implemented in this PR.**
Task `168e9561`, added to the plan by commit `d68957c` to cover Artefact 4, was explicitly *deferred* to Wave 0b.2 because it would affect `session.rs` already at the 298/300 line cap, and implementing six real quality checks requires a separate slice. **Verdict**: the PRD still describes pre-stop as part of Wave 0b. Either (a) update it to say "Artefact 4 deferred to Wave 0b.2" or (b) implement at least `(future)` scaffolding for every check. The current commit leaves the PRD lying. **Mandatory fix**.

**A2 — `actions` vs `capabilities` are still not realigned in the PRD.**
Finding A4 from v1 was marked "wont-fix with rationale", but in practice the skill (`cvg-attach.sh`), the E2E test (`e2e_two_agents_coordinate.rs`), and the setup extension now **all** use `capabilities` (the real field name in `NewAgent`). The PRD still talks about `actions`. **Verdict**: minor, but the PRD is now formally inconsistent with code in five files. **Defer with note** is acceptable if the PRD gains a half-line: "the code calls this field `capabilities` for historical reasons; the PRD calls it `actions` to flag the future rename".

**A3 — ADR-0025 status is `proposed` while the migration is already live in the PR.**
Migration `0103_system_topics.sql`, routes `/v1/system-messages`, and the E2E tests have all landed. ADR-0025 should move to `accepted` in the same PR because the contract is now *encoded*, not merely proposed. Leaving it `proposed` means CI could reasonably refuse the merge as soon as an "ADR-status-vs-implementation drift" gate exists. **Mandatory fix**: promote ADR-0025 to `accepted` as the final commit before human review.

**A4 — PRD-001 § Artefact 1 specifies heartbeat every 30s.**
The `cvg-attach.sh` skill POSTs registration and one presence message to `system.session-events`, then exits 0. No heartbeat loop. In the PRD the loop lives in "the hook agent" (Artefact 2), not in the skill. But the `.claude/settings.json` template emitted by `cvg setup agent claude` includes only `SessionStart`, not a periodic hook. **Verdict**: the heartbeat does *not* concretely run today. Say so in the commit body / CHANGELOG or add a pending Wave 0b.2 task.

**A5 — README for cvg-attach describes the "daemon offline" fallback as "warning on stderr; never blocks the user".**
True for the skill script. But `cvg setup agent claude` **does not** write a `PreToolUse` hook — so if the daemon goes down mid-session, there is no live warning. Graceful degradation exists only at `SessionStart`. The README overstates the safety net.

## B) Unsustainable promises (top 5)

**B1 — Demo script requires `cvg` with `--agents`, which is unavailable until the PR is merged.**
`demo-two-sessions.sh` has a fallback (`--help | grep` detection) that shows raw JSON when the flag is missing. Good. But the README says "if the binary in PATH does not have --agents, run `cargo install --path crates/convergio-cli --force`" — that requires compiling the workspace locally, which is non-trivial outside the dev context. For a first reader, "try the demo" is at least a three-step path. **Mitigation**: remove the "no Claude required" promise or change it to "no Claude binary required, but cvg from this PR required".

**B2 — System-message route accepts any `sender` without verification.**
`POST /v1/system-messages` accepts a body with `sender: Option<String>` and persists it. There is no cross-check between the declared sender and a registered agent. A session could publish presence for another session. For a localhost-only single-user daemon this is a reasonable policy, but an implicit promise of "authenticated agent-to-agent coordination" is not met. **Mitigation**: document explicitly in the README/ADR-0025 that the bus does *not* perform identity verification (it is single-user and trusted).

**B3 — E2E tests do not simulate a real `cvg-attach.sh` flow.**
`e2e_two_agents_coordinate.rs` calls `POST /v1/agent-registry/agents` directly with reqwest. The bash skill `cvg-attach.sh` is never exercised. A regression in the bash parser or placeholder environment would pass the test. **Mitigation**: optionally add a smoke test `tests/integration/skill-attach.sh`, or explicitly accept the gap as "shell scripts are tested by the demo".

**B4 — `cvg setup agent claude` shell-out flow is not live-tested.**
The two new smoke tests in `cli_smoke.rs` verify *that files exist* after setup. They do not verify that the `settings.json` command actually executes and registers. For an installer, the important promise is "run this pipeline → the skill works". That test is missing.

**B5 — PRD estimate 12-16 days vs this PR.**
The PR concretely delivers ADR-0025 + bus migration + skill + endpoint + setup extension + status flag + five new E2E tests + demo. Actual session time: ~2 hours of Claude work, plus human review and decisions. The PRD estimate was optimistic for pure single-developer work, but the agent-assisted reality is even more favorable. **Not a fix, a note**: the PRD can update its § Estimated effort after the PR to reflect the "with agent assistance" baseline for future estimates.

## C) Political / social / legal risks (top 3)

**C1, C2** — Already addressed by commit `d68957c` (sanitize). No new Microsoft reference or personal PID/path reference was introduced by commits `562d1e9..c008f54`. ✓

**C3 — Bash skill committed in `examples/` without shellcheck linting in CI.**
`cvg-attach.sh` and `demo-two-sessions.sh` are not covered by any gate (no shellcheck, no bats). A quoting regression (for example `${PWD}` with spaces) would slip through. The repo already has `set -euo pipefail` as convention (`best-practices.md`). Mitigation: add a minimal Wave 0b.2 task for lefthook shellcheck on `examples/skills/**/*.sh`.

## D) Metaphors that break (top 3)

**D1 — The skill "registers" the session "before any plan exists".**
"Register" suggests a formal act. What actually happens is an INSERT in SQLite with arbitrary lifetime (there is no TTL). A session registered 14 days ago and never retired (e.g. crash + machine powered off) remains "registered" forever. The term is imprecise. Mitigation: document in the README that the record is a "presence claim" that the reaper may clean up (reaper tick 60s, timeout 300s — already documented in root `AGENTS.md`).

**D2** — No new strong metaphor introduced. The metaphors "leash", "Modulor", and "ombudsman" all live outside this PR.

## E) Roadmap gaps (top 3)

**E1 — w1.4b deferred creates cascading effects on w1.9 and w1.10.**
The pre-PR review (this file) lists w1.4b as deferred. The PR can still merge with green CI, but the Wave 0b plan will not be 100% "done" after `cvg validate`. Either (a) close task `168e9561` as `failed` with reason "deferred to Wave 0b.2", or (b) leave it pending and accept that the plan itself remains non-validatable for all of Wave 0b. Operator decision.

**E2 — `cvg setup agent` for Copilot CLI emits nothing equivalent to `.claude/settings.json`.**
The PR updates only `AgentHost::Claude`. The declared principle ("Convergio above any agent") requires the same pattern for `~/.copilot/hooks/` (empty today). Natural Wave 0b.2 task.

**E3 — `system.*` topic family has no implemented retention policy.**
ADR-0025 § Retention talks about a "24h ring buffer". Nothing in the code today cleans old `system.*` messages. A Convergio instance active for months will accumulate them. Wave 0b.2 / Wave 1 task.

## F) Technical errors (top 5)

**F1** — `POST /v1/system-messages` does not reject a non-`system.*` topic at HTTP level: the bus refuses with `BusError::InvalidTopicScope`, and the error is serialized as 500 (probably — verified by test `system_message_rejects_non_system_topic`, which asserts only `is_client_error || is_server_error`). Cleaner gate: map `InvalidTopicScope` explicitly to 400. **Defer with note**: the test passes, but 500 is a generic server error, not a client error.

**F2** — `cvg agent list --output json` puts `agents` directly in the body. The response schema is undocumented. An agent consumer that calls `body.agents.len()` without the flag receives `undefined`. **Mitigation**: minimal — always add `agents: []` to JSON output so the key exists.

**F3** — `cvg setup agent claude` reads `SKILL.md` / `cvg-attach.sh` via `include_str!` using path `../../../../examples/skills/...`. If the repo structure changes (for example, workspace member renamed), `include_str!` fails at compile time. **Acceptable**: compile-time check is the right gate, but a regression test that runs `setup agent claude` in a tempdir and verifies generated file checksums would be a robust pattern.

**F4** — The `NewAgent` serde struct does not validate that `id` is non-empty and does not validate that `kind` is in a known enum. Already flagged by v0.2 task `307e6a3e` (Tighten NewAgent.kind enum + serde validation). Not blocking for Wave 0b, but because the skill posts `kind: "claude-code"`, the v0.2 plan must include `claude-code` in the enum when it lands. Inter-plan dependency.

**F5** — The merge commit body `593bda6` ("Merge branch 'main' into wave-0b") does not have conventional-commit shape. Commitlint probably does not block it (merge commits are conventionally exempt), but it is worth verifying before marking the PR ready.

## G) Verdict

**Ship now with 3 required fixes before marking the PR ready:**

1. **A1 + E1**: decide on w1.4b. Two options:
   - `cvg task transition 168e9561 failed` with message "deferred to Wave 0b.2"; update PRD `§ Artefact 4` to say "deferred"; the plan can validate with the task in `failed` (Thor accepts it as terminal).
   - Or leave the task `pending` and accept that `cvg validate` returns `fail` until Wave 0b.2 completes it. The plan remains in flight.
2. **A3**: promote ADR-0025 from `proposed` to `accepted` with a commit before PR ready.
3. **A4**: add a short note in PRD-001 on heartbeat ("loop deferred to Wave 0b.2; SessionStart-only registration is the v1 cut").

**Deferred with note** (not blocking, must not be lost):
- A2 (PRD `actions` vs code `capabilities`): add a half-line to the PRD.
- B2 (sender authenticity): document in ADR-0025.
- C3 (shellcheck on `examples/skills/`): Wave 0b.2 task.
- E2 (Copilot adapter): Wave 0b.2 task.
- E3 (system topic retention): Wave 1 task.
- F1 (map `InvalidTopicScope` → 400): defer.
- F2 (always include `agents: []` in JSON): defer.

**Wont-fix with rationale:**
- B3, B4 (testing the bash skill and shell-out installer): postponed to a shellcheck + bats pass; the Rust E2E pattern is the primary gate.
- B5 (PRD estimate): documentation that evolves over time, not a fix.
- D1 (terminology "registers"): wording cleanup.

**Estimated mandatory-fix impact**: ~30 minutes total (one PRD edit commit, one ADR-0025 promotion commit, one task transition or operator decision on w1.4b).

## Comparison with review v1

The five mandatory fixes from review v1 were addressed by commit `d68957c`. The distance between "PRD written" and "code written" narrowed significantly: this pass produced only three new mandatory fixes (vs five previously), and two of the three are decisions more than code work. The system is converging on a consistent state.
