# 2026-05-01 Triage Pass

**Operator:** Roberdan (via `claude-code-roberdan` agent)
**ADR:** [0026](../adr/0026-plan-wave-milestone-vocabulary.md)
**Friction log:** [v0.2-friction-log.md § Daemon task mirror](./v0.2-friction-log.md)
**Audit chain at start:** 729 entries, ok=true.
**Audit chain at end:** 736 entries, ok=true.

## Why

The 2026-05-01 audit of the daemon's open task list (5 plans,
~50 open tasks) surfaced ghost tasks — work that had shipped in
`main` but was still showing `pending` in the daemon — and
duplicate / stale `F##` numbering inside v0.2.

Without ADR-0026's `task.closed_post_hoc` primitive the only way
to drain these was raw SQL. With the primitive shipped (this PR),
operator triage is auditable.

## Plan renames (`Durability::rename_plan`)

Closes the lexical collision between top-level plans named `Wave …`
and the integer `wave` field on tasks inside other plans
(ADR-0026 § Decision Outcome).

| Old title | New title | Plan UUID |
|-----------|-----------|-----------|
| `Convergio Vision: Long-Tail + Urbanism (Wave 0)` | `W0 — Convergio Vision: Long-Tail + Urbanism` | `543c0d38-…` |
| `Wave 0b — Claude Code adapter (PRD-001 implementation)` | `W0b — Claude Code adapter (PRD-001 implementation)` | `2564b354-…` |
| `Wave 0b.2 — cvg session pre-stop (PRD-001 Artefact 4 deferred slice)` | `W0b.2 — cvg session pre-stop (PRD-001 Artefact 4 deferred slice)` | `db88bc17-…` |

Each rename wrote one `plan.renamed` audit row.

## Post-hoc closes (`Durability::close_task_post_hoc`)

Five tasks moved directly to `done`. Each carries a non-empty
`reason` recorded in the audit row.

| Task UUID | Title | Reason |
|-----------|-------|--------|
| `596c6601-…` | Bus poll_messages: filter own published messages | Shipped in PR #71 (F53 / ADR-0024). Wrapper Bus::poll over Bus::poll_filtered. |
| `e0eab0dd-…` | F38 — install-local.sh sync_shadowed_binary fix | Stale daemon-side numbering; same scope as friction log F44 (mirror task `75adbc93-…`). |
| `7bd232f6-…` | F39 — launchd plist must include cargo bin | Stale daemon-side numbering; same scope as friction log F45 (mirror task `232608d1-…`). |
| `307e6a3e-…` | Tighten NewAgent.kind enum + serde validation | Superseded by F52 (commit `c52a4ed`). The closed-set enum proposed here was rejected in favour of a permissive grammar so new vendors land without schema migration. |
| `2c384b19-…` | Fix: convergio-mcp + convergio-api inconsistent param name | Superseded by F52 honest correction. Schema is already consistent; F52 added explicit `_note` lines to the MCP help to prevent future agents from passing the wrong field name. |

## Wave concept restated (no schema change)

Per ADR-0026, `wave` on `tasks` is a **free-form priority bucket**.
No gate may depend on `wave 1 < wave 2`. New tasks may pick any
integer; legacy tasks keep their assigned wave. The
`wave_sequence_gate` continues to enforce its existing semantic
(no `in_progress` claim until earlier waves have at least one
task that is `done` or `failed`) — unchanged by this triage.

## What still needs work after this pass

- A proper `cvg plan triage` command (friction log F26 — daemon
  task `ce528dd3-…`) so the next pass is one CLI invocation, not
  a hand-rolled curl loop.
- The remaining `pending` v0.2 ghost candidates (`F49 — cvg graph
  estimated_tokens always returns 0`, `F49 — gh pr update-branch
  invalidates AUTO blocks`, `get_task_context auto-injects graph
  for-task pack`, `PR 14.3`, `PR 13.x` durability-split waves) —
  these are still real work, not ghosts. Left in `pending`.

## Verification

```bash
$ curl -s http://127.0.0.1:8420/v1/audit/verify
{"ok":true,"checked":736,"broken_at":null}

$ cargo run -p convergio-cli -- status --project convergio-local
# (W0 / W0b / W0b.2 visible without "Wave" prefix; ghost tasks gone.)
```
