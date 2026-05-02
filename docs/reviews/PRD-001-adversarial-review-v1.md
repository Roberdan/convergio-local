---
id: review-prd-001-v1
type: adversarial-review
target: docs/prd/0001-claude-code-adapter.md
template_version: v1
date: 2026-05-01
reviewer: claude-opus-4-7-1m (sibling-session, manual fallback per ADR-0022)
language: English
---

# Adversarial review — PRD-001 (Claude Code adapter)

> Review produced with the `docs/templates/adversarial-challenge.md` v1 template, with direct codebase verification on branch `worktree-wave-0b-claude-code-adapter` (HEAD `593bda6`, after merging `origin/main`). The author requested adversarial challenge, not reassurance.

## A) Internal contradictions (top 5)

**A1 — ADR-0025 is described as "not yet written" even though it already exists.**
PRD-001 § Bus topology says: *"that schema change is itself a small ADR (proposed but not yet written, will be ADR-0025 if accepted as part of this PRD)"* (`docs/prd/0001-claude-code-adapter.md:147-148`). However, `docs/adr/0025-system-session-events-topic.md` already exists (285 lines, status `proposed`). **Reality prevails**: remove "not yet written" or rephrase it as "drafted alongside this PRD".

**A2 — Installer naming diverges.**
PRD-001 § Risks says: *"ship a one-line installer (`cvg setup claude-code`)"* (`docs/prd/0001-claude-code-adapter.md:303-304`). The real binary exposes `cvg setup agent claude` (`crates/convergio-cli/src/commands/setup.rs`, host enum value `Self::Claude => "claude"`). These are two different API names. **The code already on `main` prevails** — realign the PRD.

**A3 — Heartbeat "background loop" inside a one-shot hook.**
§ Artefact 2 says: *"Heartbeat: a background loop in the hook agent runs `POST /v1/agents/:id/heartbeat` every 30 seconds"* (lines 118-119). Claude Code hooks are one-shot commands executed by the harness, not long-running processes. The PRD does not explain who keeps the loop alive after the hook returns. Possible interpretations: (a) launchd plist, (b) a separate daemon helper, (c) reaper-driven behavior in the daemon itself. Make this explicit.

**A4 — `actions` vs the real `NewAgent` schema.**
§ Artefact 1 specifies a payload with `kind`, `name`, `host`, `actions`, and `metadata` (lines 73-86). The pending v0.2 plan task *"Tighten NewAgent.kind enum + serde validation (was: undocumented, accepts garbage)"* (task `307e6a3e`) implies that `NewAgent` exists but its fields are not yet official. The PRD assumes a contract the current code **does not validate**: a session could POST `actions: ["delete-prod", "rm-rf"]` and the daemon would accept garbage. The previous review already named this gap. Update the PRD to declare the order: strict enum first, adapter second.

**A5 — Drift between root `AGENTS.md` and `/v1/agent-registry/*`.**
Root `AGENTS.md` § "MCP tools available" lists only `/v1/agents/spawn` and `/v1/agents/:id/heartbeat`. The code (`crates/convergio-server/src/routes/agent_registry.rs`) has four distinct `/v1/agent-registry/agents*` endpoints. The root file does not mention them. This is not the PRD's fault, but it becomes a visible inconsistency as soon as a third agent reads both documents together.

## B) Unsustainable promises (top 5)

**B1 — "Same skill pattern, separate PRDs, Wave 2".**
§ "What this PRD does *not* deliver" promises Cursor / Codex / Copilot adapters in Wave 2 with the *same* skill pattern (`docs/prd/0001-claude-code-adapter.md:285-287`). Wave 2 in the ROADMAP is four weeks. Four vendors × installer + skill template + per-vendor hook semantics is optimistic. The promise is useful if motivated by urbanism (marginal extensibility), but it needs a concrete margin: "Wave 2 ships *one* additional vendor as proof; the others remain roadmap work until dogfood feedback".

**B2 — Hook latency "single-digit ms locally".**
§ Risks claims *"response is 200 in single digit ms locally"* (line 298). No benchmark is attached. A local HTTP call through curl is often 5-15 ms RTT on macOS, not reliably single digit. Add telemetry from day one (already referenced in Estimated effort as "2 days — telemetry"), but without a measured baseline the mitigation is optimistic.

**B3 — `cvg pr sync <plan_id>` as a suggested action.**
§ Artefact 4 check 1 recommends: *"Suggested action: `cvg pr sync <plan_id>` (T2.04 integration)"* (line 173). The `cvg pr` binary exposes only `stack`. There is no `sync`. T2.04 is not verifiable as a referenceable task. Promising an action the PRD does not implement, and that has no PRD/ADR home, is documentation debt.

**B4 — `cvg bus ack <message_id>` as a suggested action.**
Same pattern: § Artefact 4 check 2 (line 174). There is no top-level `cvg bus` subcommand (`cvg --help` does not list it). Today the bus is reachable only via HTTP `POST /v1/messages/:id/ack`. PRD-001 must either admit that and suggest the curl call, or commit to making `cvg bus ack` part of the artefacts.

**B5 — 12-16 day estimate for a single developer.**
Plausible under full focus. However, PRD-001 itself admits the developer is also the author of VISION/ADR/Wave 0a and maintains cross-corporate context (see C). Twelve to sixteen days becomes roughly three calendar weeks and looks optimistic once context switching is counted. Add a high range of 4-5 weeks for honesty.

## C) Political / social / legal risks (top 3)

**C1 — Explicit reference to "Microsoft alignment story (ADR-0017)".**
§ "Why now" lists *"Microsoft alignment story (ADR-0017) needs a working demo"* (lines 55-58) and cites "ISE Engineering Fundamentals". The PRD becomes a committed document in a public repo. Corporate employer IP review processes can read `Microsoft alignment` as unauthorized endorsement even when the reference is soft. Mitigation: move the motivation to an internal comment or rephrase it as "alignment with industry-standard engineering principles".

**C2 — "The operator just lived the failure".**
§ "Why now" cites real PIDs (`5424`, `77685`) and a user path (`/Users/Roberdan/GitHub/convergioV3`) as evidence from that morning (lines 32-38). This is useful evidence practice but leaks personal setup details in a public-facing document. Mitigation: paraphrase as *"two concurrent Claude Code sessions in the same repo"* without personal paths.

**C3 — Hard-coded `kind: "claude-code"` string.**
§ Artefact 1 POSTs `kind: "claude-code"`. This is an identifying string for a third-party commercial product (Anthropic). Convergio is an OSS project under the Convergio Community License v1.3. Hard-coding the vendor name in an official payload (a) binds the contract to one product and (b) creates precedent for `claude-desktop`, `claude-api`, and similar variants without criteria. Mitigation: server-side enum + ADR justifying the vocabulary; the missing enum is already flagged by v0.2 task `307e6a3e`.

## D) Metaphors that break (top 3)

**D1 — "Traffic officer does not sign the certificate of habitability".**
§ Artefact 4 (line 152) uses the traffic-officer metaphor to justify `cvg session pre-stop`. The metaphor is elegant but overwhelms a mechanism that is simply *a session-end consistency check*. A traffic officer does not refuse a certificate at discretion; they follow regulated checklists. The metaphor suggests discretion that the check does not have. Rephrase as *"end-of-day audit"* or *"cleanup gate"* — less romantic, more honest.

**D2 — "Long-tail thesis (ADR-0016)" as the driver for Wave 0.**
§ Problem cites ADR-0016 as the rationale (*"a shovel that does not coordinate parallel diggers is a single-user tool"*, line 41). For a pragmatic technical reader (CI bot, future maintainer in six months), this is marketing language. The concrete motivation — "two sessions see their own commits but not each other's progress" — is already in the PRD and is enough. The long-tail citation is "why to sell", not "why to build".

**D3 — "Convergio is the leash for any AI agent".**
Root `AGENTS.md` opens with this phrase. It is a strong image and old in the codebase, not introduced by PRD-001. But "leash" suggests unilateral constraint: the owner pulls the agent. Convergio's reality is cooperative (server-side gates, but clients are cooperative and an uninstrumented client can bypass everything). PRD-001 should at least state that the constraint is cooperative, not forced. Otherwise a first reader expects enforcement that does not exist.

## E) Roadmap gaps (top 3)

**E1 — `cvg setup agent claude` already exists, but the PRD ignores it.**
Commit `85332ea` (29 April, co-authored by Copilot) added the subcommand. The PRD talks as if `cvg setup claude-code` must be built from scratch. **Effect**: the executor of task w1.5 may write a new setup command instead of extending the existing one. Update the PRD to acknowledge the skeleton and define the extension (generate `.claude/settings.json` in addition to `mcp.json`).

**E2 — Wave 0b plan duplicates task w1.6 with drift from PR #58.**
Plan v0.1.x has task `9ce7a17c` (`cvg status v2: human-friendly progress dashboard`), closed by PR #58. Wave 0b has task w1.6 (`cvg agent list flag + EN/IT i18n`), which should extend the `status_render` introduced by #58. The PRD does not mention the piggyback. Without an explicit note, the w1.6 executor will rebuild the structure.

**E3 — Heartbeat 30s vs watcher tick 30s vs reaper tick 60s.**
The PRD prescribes heartbeat every 30s (line 118). The daemon watcher loop runs every 30s (`CONVERGIO_WATCHER_TICK_SECS` default); the reaper runs every 60s with timeout 300s. If hooks fail repeatedly and the daemon misses two consecutive heartbeats, the watcher may flip state before the reaper releases leases. Effect: spurious lease lock-out. The PRD must declare the `(heartbeat_interval, reaper_timeout, watcher_threshold)` window and why it is stable.

## F) Technical errors (top 5)

**F1** — Endpoint `/v1/agent-registry/agents` is referenced correctly (PRD lines 71-72) — verified in `crates/convergio-server/src/routes/agent_registry.rs:13`. ✅ OK.
**F2** — `cvg setup claude-code` does not exist; the real command is `cvg setup agent claude`. See A2.
**F3** — `cvg pr sync <plan_id>` does not exist; `cvg pr` only has `stack`. See B3.
**F4** — `cvg bus ack <message_id>` does not exist; there is no `cvg bus` subcommand. See B4.
**F5** — `cvg session pre-stop` does not exist; `cvg session` only has `resume`. It must be created as part of Artefact 4 (the PRD says this implicitly, but the corresponding w1.x task is not clear in the Wave 0b plan: none of the 10 tasks is named "implement cvg session pre-stop"). **Plan-PRD drift**.

## G) Verdict

**Ship now with 5 required fixes before the first new code commit:**

1. **A1**: remove "ADR-0025 not yet written" from the PRD (it is already written).
2. **A2/F2**: realign the PRD to existing `cvg setup agent claude` and state that w1.5 *extends*, not *creates*.
3. **B3, B4, F3, F4**: for every `cvg <verb>` cited in the PRD but absent, choose one: (a) implement it as part of Wave 0b and add the corresponding plan task, (b) replace it with the curl/HTTP call, or (c) mark it `(future)`.
4. **F5/Plan drift**: add an explicit Wave 0b plan task for `cvg session pre-stop` or document which of the 10 existing tasks covers it.
5. **C1, C2**: sanitize `Microsoft alignment` references and personal PID/path references in the public PRD.

**Deferred with note** (not blocking Wave 0b, but must be revisited):
- A3 (heartbeat loop): document the mechanism in a mini-ADR or PRD update. Do not block the first skill ship.
- D1, D2, D3 (metaphors): wording cleanup; revisit in Wave 1 if dogfood demands it.
- E3 (heartbeat/reaper window): add a "timing" section to the PRD if Artefact 4 requires it.

**Wont-fix with rationale**:
- A4 (`NewAgent.kind` enum): an independent v0.2 task, not a cause to block Wave 0b. Coordinate in the plan but do not block here.
- A5 (root `AGENTS.md` drift): a root-file bug to fix separately in a docs PR.

**Estimated fix impact**: one additional person-day, mostly prose edits in the PRD and 2-3 plan tasks. Nothing changes in the 12-16 engineering-core days.
