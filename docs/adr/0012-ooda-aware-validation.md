# 0012. OODA-aware validation: outcome reliability over output reliability

- Status: accepted
- Date: 2026-04-30
- Deciders: Roberdan, office-hours dogfood session
- Tags: layer-4, validation, agent-orchestration, vision

## Context and Problem Statement

ADR-0011 made Thor the only path from `submitted` to `done`. That
turned the validator into the gate every agent has to traverse — but
it left Thor as a thin gate: it checks `evidence_required` kinds are
present, nothing more. A plan whose tasks all report "test_pass"
evidence with `exit_code: 0` validates as Pass, even if the project's
real test suite is broken. Convergio's claim is that the leash holds.
A leash that only checks evidence *shape* is half a leash.

Three concrete user concerns surfaced during the office-hours session
that this ADR captures collectively:

1. **Thor context.** Today Thor sees `(plan_id, list of tasks,
   evidence_kinds)`. It does not see the actual evidence payload
   content, the touched crate's `AGENTS.md`, related ADRs, or
   historical refusals on the same pattern. A smart validator needs
   that context.
2. **Memory + learning.** The audit chain is already a memory of
   every refusal. Nobody reads it back. The system repeats the same
   refusals because nothing aggregates the past.
3. **Negotiation, not just refusal.** Today an agent can only retry
   after a refusal. There is no structured way to say "the plan was
   wrong, here is the better thing I did, here is why." Without a
   negotiation channel, agents loop forever or burn out the user.

A fourth concern — about agent orchestration (long runs, model
routing, multi-vendor support) — is in scope for this ADR as the
*context* in which OODA validation runs, but the implementation
mechanics for orchestration live in their own follow-up tasks
(T4.01–T4.05).

## Decision Drivers

- **Outcome > Output.** Convergio targets reliable *outcomes* (the
  user's user gets working software), not reliable *outputs* (the
  agent produced something it claims is done). A validator that
  cannot distinguish "tests pass shape" from "tests actually pass"
  ships outputs, not outcomes.
- **OODA loop.** Agent + Thor + human form an Observe-Orient-Decide-
  Act triangle. The agent acts on its plan; Thor observes the
  evidence; one or both orient on real-world signal; the human
  decides when they cannot converge. This is the same loop every
  reliable execution system uses (aviation, surgery, air-traffic
  control). It maps cleanly onto Convergio's layers.
- **Symmetry.** If the agent must respect Thor's refusal, Thor must
  respect the agent's plan amendments. Asymmetric protocols breed
  rebellion; symmetric protocols breed collaboration.
- **Boundedness.** No loop runs forever. After N rounds without
  convergence, the human is the next OODA actor.

## Considered Options

### Option A — Stop at ADR-0011

Thor sets `done`. Smart validation, learning, negotiation, and
escalation are deferred indefinitely. The "leash" claim stays
half-fulfilled.

### Option B — Build everything at once

One large change that delivers smart Thor + learnings store +
negotiation protocol + 3-strike escalation in a single PR. High
coordination cost, slow to ship, hard to review.

### Option C — Layered roadmap with this ADR as the spine (chosen)

Document the full OODA-aware validation model now. Implement it
incrementally through plan tasks T3.02 (smart Thor), T3.03
(negotiation), T3.04 (escalation), T4.01 (context packet), T4.02
(learnings store), T4.03 (reputation), T4.04 (multi-vendor),
T4.05 (long-run sessions). Each piece lands as its own PR, but
every PR can point at this ADR for the why.

## Decision Outcome

Chosen option: **Option C**, because the model is too large to land
in one PR and too important to leave undocumented while we
incrementally deliver it.

### The OODA validation model

Each task transition through Thor follows four steps, mapped to the
OODA loop:

```text
Agent submits        →  OBSERVE   Thor receives evidence + context packet
                                  (T4.01: AGENTS.md, related ADRs, payload,
                                  past refusals, repo state)
                     ↓
Thor validates       →  ORIENT    runs the project pipeline (T3.02: cargo
                                  fmt, clippy -D warnings, test, doc-checks,
                                  ADR coherence, link checker), consults the
                                  learnings store (T4.02), checks the
                                  agent's reputation (T4.03)
                     ↓
                     →  DECIDE    Pass / Fail / NeedAmendment
                     ↓
On Pass              →  ACT       complete_validated_tasks promotes
                                  submitted -> done; learnings store
                                  records the success.
On Fail              →  ACT       audit row task.refused; learnings store
                                  records the pattern; agent receives a
                                  structured refusal that names the gate,
                                  the reason, AND the closest historical
                                  fix from the learnings store.
```

When the agent disagrees with Thor's refusal — when in fact "the plan
was wrong and I did better" — the agent uses the **negotiation
channel** (T3.03):

```text
agent  -> propose_plan_amendment {
            task_id, rationale, replacement_acceptance_criteria,
            evidence_diff
          }
Thor   -> Accept | Reject | NeedHumanReview
```

After **3 unsuccessful rounds** on the same task (3 refusals or 3
rejected amendments — T3.04), Convergio writes
`task.escalation_required` to the audit chain and publishes a
plan-scoped bus message naming the human (the plan owner). The human
is the next OODA actor: they read the rounds, decide, and either
override (audit row `task.human_override` with reason text) or
restructure the task.

### Memory and learning

Every Thor decision is already in the audit chain. The new
**learnings store** (T4.02) is a query view over `audit_log`:

```sql
SELECT
  payload->>'gate'   AS gate,
  payload->>'reason' AS reason_pattern,
  COUNT(*)           AS refusal_count,
  MIN(created_at)    AS first_seen,
  MAX(created_at)    AS last_seen
FROM audit_log
WHERE kind = 'task.refused'
GROUP BY gate, reason_pattern;
```

Surfaced via `GET /v1/audit/learnings`. Thor reads this before
emitting a refusal so it can append "this pattern has been refused N
times in the last M days; the closest acceptance after a fix used
diff-shape X" to the response. The same data feeds an
`agent_reputation` view (T4.03) that the planner consults when
routing tasks (T4.04).

### Reward / punishment as observable signal

"Reward" is `task.completed_by_thor`. "Punishment" is
`task.refused`. Both are already audit kinds. The new pieces are:

- per-agent aggregation (`agent_reputation`),
- per-pattern aggregation (the learnings store),
- a planner that uses both signals when assigning waves to agents.

This is not RLHF in any meaningful sense. It is a feedback loop
backed by a tamper-evident log. The system "learns" in the sense
that agents who repeat the same refusal lose routing priority; the
learnings store also surfaces the refusal pattern verbatim to the
agent's prompt so it has the *information* to do better.

### Long runs, context, and model routing (T4.04, T4.05)

Out of scope for the validator itself, but in scope for the OODA
context:

- **Long-run sessions** (T4.05): durable `agent_session` rows so an
  agent can rehydrate its context after a host restart. Today the
  daemon survives a host restart but the agent does not.
- **Model routing** (T4.04): the planner consults `agent_capabilities
  × cost × reputation` and routes a wave to the cheapest agent that
  passes the capability filter and has a high-enough reputation.
  Multi-vendor adapters (Codex via `codex` CLI, GPT via Copilot CLI,
  Cursor, Cline, Continue) plug into the same agent_registry surface
  that already exists. Each declares cost ($/Mtok), latency target,
  and capabilities at registration.
- **Context budget** (CONSTITUTION § 13): an agent on Convergio works
  in chunks ≤ 5_000 LOC where possible, so its window stays warm for
  the OODA loop above.

### Positive consequences

- Validation goes from "evidence shape check" to "outcome
  verification". The leash claim becomes mechanical.
- Refusals become useful: the agent receives a structured pointer to
  the learnings store, not a blank "no".
- Disagreement gets a channel. The agent can be right. Convergio
  treats the negotiation as data, not insubordination.
- Loops terminate. After 3 rounds the human owns the next decision.
- Multi-vendor + cost-aware routing become first-class. Convergio
  becomes the orchestration substrate, not a Claude-only loop.

### Negative consequences

- Substantial implementation surface. Mitigated by sequencing the
  pieces as plan tasks T3.02 (smart Thor), T3.03 (negotiation),
  T3.04 (escalation), T4.01–T4.05.
- Each new audit kind (`task.amendment_proposed`,
  `task.amendment_accepted`, `task.amendment_rejected`,
  `task.escalation_required`, `task.human_override`) extends the
  vocabulary the audit verifier and the bus consumers must know.
  Backward-compatible additions only — no audit row format change.
- Smart Thor that runs the pipeline takes minutes, not seconds. The
  validator becomes asynchronous. `cvg validate` should optionally
  return a `verdict_id` and let the caller poll. T3.02 must be
  designed for async-by-default.

## Pros and Cons of the Options

### Option A (stop at ADR-0011)

- 👍 Zero further work. Today's leash is already better than
  v0.1.0.
- 👎 The README claim ("a local daemon that refuses agent work
  whose evidence does not match the claim of done") is true only at
  the surface. A broken-pipeline plan still validates. Outside users
  can demonstrate the gap in 5 minutes.

### Option B (one big PR)

- 👍 Single coherent change.
- 👎 Unreviewable size. Long-running branch. Conflicts pile up.
  Violates CONSTITUTION § 13 (agent context budget) — the PR itself
  exceeds an agent's review window.

### Option C — layered roadmap (chosen)

- 👍 Each piece is reviewable and individually useful.
- 👍 Failures revert without losing the rest.
- 👍 The vision is documented as one ADR even though the
  implementation arrives in pieces — future agents reading any one
  PR can find the trajectory.
- 👎 Discipline cost: the project must keep this ADR in sync as the
  pieces land. Mitigated by referencing T-numbers; PRs that close
  T-tasks naturally update the ADR's status.

## Out of scope

- Concrete schema for the `agent_amendment` table — designed in T3.03.
- Exact aggregation SQL for the learnings store — designed in T4.02.
- The 3-strike threshold value (3 rounds is a starting point; T3.04
  may make it configurable per plan).
- Any changes to the existing gate pipeline beyond integration with
  the new context packet — gates remain authoritative, defense in
  depth.

## Links

- [ADR-0011](0011-thor-only-done.md) — Thor as the only path to
  `done`.
- [ADR-0007](0007-workspace-coordination.md) — the workspace coordination
  primitives this ADR builds on.
- [CONSTITUTION § 6](../../CONSTITUTION.md) — clients propose, daemon
  disposes.
- [CONSTITUTION § 13](../../CONSTITUTION.md) — agent context budget.
- Office-hours plan tasks T3.02, T3.03, T3.04, T3.06, T3.07,
  T4.01–T4.05 on plan `8cb75264-8c89-4bf7-b98d-44408b30a8ae`.
- Friction log finding F13: agent-driven `done` (closed by ADR-0011).
- The OODA loop: John Boyd, USAF, 1976. Origin in fighter pilot
  decision cycles; standard reference in execution-critical systems.
