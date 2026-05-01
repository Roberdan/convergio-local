# Agent resume packet

**This is the file a fresh AI agent should read first** when handed
this repository. It is paste-ready: every command line below works
verbatim against the running daemon and the current `cvg` binary.

The packet is the canonical answer to the question *"a previous
session ended; how do I pick up where it left off without burning
context on archaeology?"*.

It was validated on 2026-04-30 — a fresh sub-agent armed with this
packet (early draft) opened PR #32 in 25 minutes from cold context.
The four findings from that test (F29-F32) are folded back into
this version. See
[`docs/plans/v0.2-fresh-eyes-test-result.md`](./plans/v0.2-fresh-eyes-test-result.md)
for the full report.

---

## 1. Identity

You are operating on a Mac at `/Users/Roberdan/GitHub/convergioV3`.

The Convergio daemon (`v0.1.2` running, target `v0.2.0` after
release-please PR #18 merges) is at `http://127.0.0.1:8420` and is
the **source of truth** for plans, tasks, evidence, and the
hash-chained audit log. If the daemon is down, start it:

```bash
cvg service start
cvg health    # expect ok=true, service=convergio, version=0.1.x
```

Your durable agent identity in `agent_registry` is
`claude-code-roberdan`. Use it on every transition:

```bash
cvg task transition <task_id> in-progress --agent-id claude-code-roberdan
```

If the registry has lost the row (e.g. fresh DB), re-register:

```bash
curl -fsS -X POST http://127.0.0.1:8420/v1/agent-registry/agents \
  -H 'Content-Type: application/json' \
  -d '{
    "id": "claude-code-roberdan",
    "kind": "claude",
    "name": "Claude Code (Roberdan local)",
    "host": "macOS",
    "capabilities": ["code","test","doc","rust","bash","markdown"]
  }'
```

## 2. Cold-start reads (in order)

Live state first — every value below is a daemon query, never stale:

```bash
cvg session resume                       # daemon, audit, active plan, next tasks, open PRs
cvg session resume --output json         # same brief, machine-readable
cvg pr stack                             # merge order + conflict matrix (uses gh)
git log --oneline main -10               # what landed recently
```

Then the timeless reference set:

```bash
cat AGENTS.md            # cross-vendor agent rules
cat CONSTITUTION.md      # 16 non-negotiables (§ 6, § 11, § 13, § 15, § 16, P5)
cat ROADMAP.md           # priorities v0.2.x → v0.3 → v0.4+
cat docs/INDEX.md        # auto-generated file map
cat docs/plans/v0.2-friction-log.md           # accumulated frictions
```

## 3. Worktree discipline (CONSTITUTION § 15)

If another agent might be operating on this repo at the same time,
work from a separate git worktree. Single-checkout
`git checkout` switching is reserved for genuinely solo sessions.

```bash
# Worktrees live under .claude/worktrees/<branch>/, gitignored AND
# excluded from cross-vendor agent context (.claudeignore /
# .cursorignore / .github/copilot-ignore). They do not show up in
# `git status`, do not pollute editor search, and do not consume
# context windows.
git worktree add .claude/worktrees/<branch-name> -b <branch-name>
cd .claude/worktrees/<branch-name>

# work, commit, push as usual
gh pr create --base main --head <branch-name> --title "..." --body "..."

# at end of work
cd /Users/Roberdan/GitHub/convergioV3   # back to main checkout
git worktree remove .claude/worktrees/<branch-name>
```

## 4. Workspace lease pattern (claim before edit)

When you are about to edit a file, especially one that other agents
might race against, claim a workspace lease. Hold for an hour, then
release.

```bash
EXPIRES=$(date -u -v+1H +%Y-%m-%dT%H:%M:%SZ 2>/dev/null \
       || date -u -d "+1 hour" +%Y-%m-%dT%H:%M:%SZ)

LEASE_ID=$(curl -fsS -X POST http://127.0.0.1:8420/v1/workspace/leases \
  -H 'Content-Type: application/json' \
  -d "{
    \"resource\": {\"kind\":\"file\",\"project\":\"convergio-local\",\"path\":\"$FILE\"},
    \"agent_id\": \"claude-code-roberdan\",
    \"purpose\": \"$WHY\",
    \"expires_at\": \"$EXPIRES\"
  }" | jq -r .id)

# ... edit the file ...

curl -fsS -X POST "http://127.0.0.1:8420/v1/workspace/leases/$LEASE_ID/release" \
  -H 'Content-Type: application/json' -d '{}'
```

For a solo session this is overhead you may skip; the lease pattern
exists for the multi-agent future (T4.04).

## 5. Required local pipeline before any push

```bash
cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" cargo clippy --workspace --all-targets -- -D warnings
RUSTFLAGS="-Dwarnings" cargo test --workspace
./scripts/check-context-budget.sh   # exit 0 clean, 2 soft-warn ok
./scripts/generate-docs-index.sh --check
./scripts/legibility-audit.sh --quiet  # target ≥ 70, ideal ≥ 85
```

If any step fails, fix first, then re-run **all** of them. Never
push with known failures.

## 6. PR template hygiene (CONSTITUTION § 13 + F23)

Every PR body has six sections:

```markdown
## Problem
## Why
## What changed
## Validation
## Impact
## Files touched
```

The `Files touched` block lists paths produced by:

```bash
git diff --name-only main...HEAD
```

The path strings must match exactly. `cvg pr stack` cross-checks
the manifest against the real diff and surfaces `Mismatch` /
`Missing` if you got it wrong.

## 7. WIP commit message protocol (F29 + F30)

If you must pause work, commit it as a `wip(...)` and push the
branch. The commit body must include:

- a list of files modified, each with current `wc -l`
- new modules added: their `pub mod ...` declaration line
- the resume checklist with each remaining sub-step
- the canonical resume command:
  ```bash
  git checkout <branch>
  git rebase origin/main
  ```

A future T1.20 ships this as `docs/wip-commit-template.md`.

## 8. Constitution touchstones

| § | What it says | Common mistake |
|---|--------------|----------------|
| § P5 | i18n first — strings flow through Fluent | new CLI command shipped EN-only |
| § 6 | clients propose, daemon disposes; only Thor sets `done` | calling `cvg task transition X done` (clap blocks at parse) |
| § 11 | every crate has AGENTS.md + CLAUDE.md | new crate shipped without one |
| § 13 | per-file 300 lines, per-crate 5/10k LOC | new file lands at 301 lines |
| § 15 | parallel-agent work uses worktrees | one shared checkout, two agents |
| § 16 | legibility score ≥ 70 / 100 | regression during a busy PR wave |

## 9. The first wave for a new session

The user's standing ask is that any new session opens with a repo
optimisation pass — make the codebase more legible for the next
agent before adding new surface. The concrete queue is **not**
listed here on purpose: it goes stale, and the daemon already knows
the order.

```bash
cvg session resume     # next-priority pending tasks, ordered by wave/sequence
```

The principle that shapes the queue is constant:
- *Housekeeping first* — install-script, hooks, locale pins, WIP protocol.
- *Then retrieval* — frontmatter, coherence checks, file-map quality.
- *Then architecture* — splitting near-cap crates (durability is
  the standing candidate; check `./scripts/legibility-audit.sh` for
  the current LOC).

Run wave 1 tasks in `wave/sequence` order; check the legibility
score after each PR to see the trend.

## 10. After the optimisation wave

The next strategic milestone is **smart Thor** (T3.02): the
validator runs the project's actual pipeline (`cargo fmt`,
`clippy -D warnings`, `cargo test --workspace`, doc-coherence,
ADR-link check) before emitting `Pass`. ADR-0012 is the spine. The
plan tasks T3.02-3.07 + T4.01-4.05 detail every layer.

## 11. What to do when stuck

1. Stop. Do not guess your way through.
2. Check `cvg status --project convergio-local` — the queue is the
   single source of truth for "what is open".
3. Read the most recent friction log
   (`docs/plans/v0.2-friction-log.md`) — your problem is probably
   already named there.
4. If genuinely new, capture it as a new finding (next number after
   F32) and continue.
5. If a hard architectural fork emerges, write an ADR draft at
   `docs/adr/00NN-<title>.md` (status `proposed`) and stop until
   the user reviews.

The audit chain accepts every refusal. Convergio's loyalty is to
the truth, not to the agent's pace.
