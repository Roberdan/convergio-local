# WIP commit template

When you must pause unfinished work and hand the branch over to
another agent (or to your own future self after a context reset),
the WIP commit message is the bridge. It must contain enough
mechanical detail that the next agent can resume without rereading
the conversation that produced the work.

This template is the *canonical pause-checklist*. It exists because
two real handoffs failed without it (F29, F30): a fresh sub-agent
could not infer that `pr_diff.rs` was added but not declared in
`commands/mod.rs`, and that `pr.rs` had landed at 301 lines (1 over
the 300-cap) so the next change had nowhere to grow.

---

## When to use

- You are stopping mid-task with code that does not yet build,
  test, or pass clippy.
- The branch must survive a session reset / agent handoff.
- Trivial single-line WIPs do not need this template; use it when
  the diff is non-obvious.

## Canonical message

Subject line:

```
wip(<crate>): <one-line summary> — paused
```

Body, in order:

```text
## Files modified
- crates/foo/src/bar.rs            (148 lines)
- crates/foo/src/bar/inner.rs      (NEW, 82 lines)
- crates/foo/src/baz.rs            (301 lines, OVER 300-cap — needs split before unpause)

## New module declarations required
- crates/foo/src/lib.rs needs `pub mod bar;` (added in this commit, verify after rebase)
- crates/foo/src/bar/mod.rs needs `mod inner;` (NOT yet added — block until done)

## Build / test state at pause
- cargo check: PASS
- cargo clippy: FAIL (1 warning in bar.rs:42 — fix before unpausing)
- cargo test --workspace: NOT RUN

## Resume checklist (next agent / next session)
1. Split `baz.rs` along the `widget` / `gadget` seam (target ≤ 250 lines each).
2. Add `mod inner;` to `bar/mod.rs`.
3. Fix the clippy warning in bar.rs:42 (use `let _ =` or document why).
4. Run `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.
5. Run `./scripts/check-context-budget.sh` and `./scripts/legibility-audit.sh --quiet`.
6. Open PR with the standard 6-section template.

## Resume command

```bash
git checkout <branch>
git rebase origin/main
```
```

## Rationale for each block

- **Files modified with `wc -l`**: lets the next agent see whether
  any file is at or over the 300-cap before they touch it. Saves a
  surprise pre-commit failure mid-rebase.
- **New module declarations**: every new `*.rs` file needs a `mod X;`
  or `pub mod X;` in the parent. The compiler's error here is local
  to a different file from the one you added; an agent reading only
  the new file will miss it.
- **Build / test state**: tells the next agent exactly which gate
  blocks unpause. Without this they re-run everything to find out.
- **Resume checklist**: the smallest set of actions that flips the
  branch from `paused` to `submittable`. Numbered, ordered, no
  context required.
- **Resume command**: removes ambiguity about whether to rebase,
  merge, or cherry-pick. The repo policy is rebase on `origin/main`.

## Where this template fits

`scripts/install-local.sh` (T1.21) installs `lefthook` so the
file-size guard catches the 300-cap regression on commit. The WIP
template is the human/agent companion: even when the gates pass at
pause time, the message captures the *implicit state* (planned
changes, near-cap files) that the gate cannot see.

## Anti-patterns

- "WIP — will fix later" with no body. The next agent does not have
  the conversation that produced the work.
- A body that describes intent but not file paths or line counts.
  Intent rots faster than mechanical state.
- Deleting the WIP commit during rebase. Never. Squash-merge would
  also delete it; this repo runs merge-commit only for that reason.
