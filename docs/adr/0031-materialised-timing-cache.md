---
id: 0031
status: accepted
date: 2026-05-03
topics: [layer-1, durability, telemetry, dashboard]
related_adrs: [0003, 0011, 0026, 0029]
touches_crates: [convergio-durability, convergio-tui]
last_validated: 2026-05-03
---

# 0031. Materialised timing cache + plan↔PR link table

- Status: accepted
- Date: 2026-05-03
- Tags: durability, telemetry, dashboard

## Context

`cvg dash` drill-down (PR #114 / #116) shows a plan's tasks but
cannot show how long each one took, when it started, or which PRs
it shipped through. The information exists — every transition
writes an audit row, and PRs reference tasks via `Tracks: T<id>` —
but reconstructing it for every render means joining `audit_log`
on every dashboard tick. That is fine for one operator and one
plan; it does not scale once there are five concurrent agents and
a 30-task plan.

`cvg session pre-stop check.plan_pr_drift` (plan db88bc17, deferred
slice of PRD-001 Artefact 4) needs the same plan↔PR mapping but
written, not heuristically derived from PR titles.

## Decision

Two narrow additions, both in `convergio-durability`:

1. **Migration 0009 — task / plan timing columns.** Three new
   columns each on `tasks` and `plans`:
   - `started_at TEXT` (RFC3339 of the first `in_progress`
     transition; NULL until then),
   - `ended_at TEXT` (RFC3339 of the most recent transition into
     `done`/`failed`/`cancelled`),
   - `duration_ms INTEGER` (ended − started, NULL until ended).

   Written by `Durability::transition_task` and `transition_plan`
   in the **same transaction** as the audit row. The audit log
   remains the source of truth; the columns are a derived cache
   that never disagrees with it because both writes are atomic.

2. **Migration 0010 — `plan_pr_links` table.** Canonical
   plan↔PR mapping with `repo_slug`, `pr_number`, optional
   `task_id` and `branch`. Written by a future
   `POST /v1/plans/:id/pr-links` (and the matching
   `cvg pr link <plan> <#>` CLI). Until that ships, the dashboard
   keeps falling back to the title/branch heuristic with a
   `· no link` crumb so the missing data is honest, not invented.

## Consequences

- The existing `transition_task` audit row still says everything —
  `from`, `to`, `at`, `agent_id`. The cache column write is an
  additional `UPDATE` inside the same `BEGIN/COMMIT` so it is
  invariant under the audit chain (Constitution P1: zero tolerance
  for derived state diverging from the source of truth).
- Dashboard renderers may now read `tasks.duration_ms` directly
  without a join. The detail overlay can show "ran for 14m 3s" on
  every drill without recomputing.
- `plan_pr_links` enables future `cvg pr stack` plan-aware grouping
  and the `check.plan_pr_drift` step of `cvg session pre-stop`.
- Migration version space stays in the durability range (1-100,
  per ADR-0003).

## Alternatives considered

- **Reconstruct on every render** — what we do today. Cheap to
  implement, expensive at scale. Rejected once the dashboard ran
  with 30+ active plans.
- **Separate `task_timing` table** — gives a clean boundary but
  adds a join for the most common query (one task's duration).
  Rejected for the same reason audit rows live next to the row
  they describe: the typical access pattern is "row + its time".
- **Wall-clock from outside the transaction** — would let the
  audit row and the cache disagree by milliseconds in the worst
  case. Rejected because Constitution P1 is non-negotiable.
