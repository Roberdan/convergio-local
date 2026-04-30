# STATUS — pointer to live state

> **For current state, run `cvg session resume`.**
> It prints daemon health, audit chain integrity, the active plan,
> the next-priority pending tasks, and open PRs — all queried live
> from the daemon, never stale. JSON: `cvg session resume --output json`.

This file used to be a hand-written snapshot. It went stale within
hours. The daemon is the source of truth; this page now exists only
to point you at the right command.

## In one paragraph

Convergio is a local-first daemon that **refuses agent work whose
evidence does not match the claim of done**. The gate pipeline,
hash-chained audit, and `cvg` CLI ship together; the validator
(Thor) is the only path that promotes a task to `done`. The product
is eaten by its own users — Convergio is built while building
Convergio.

## Direction

> "Build Convergio while building Convergio. Each round you learn
> first-hand, you find what works, what doesn't, and you improve."

The legibility score, `docs/INDEX.md`, worktree discipline, and the
single durable plan are the controls keeping the project from
becoming what v2 became: too big to follow.

## How to navigate this repo as an agent

1. Run `cvg session resume` for live state (this is your cold-start).
2. Read [`docs/agent-resume-packet.md`](./docs/agent-resume-packet.md)
   for the timeless protocol (worktree, lease, pipeline, constitution
   touchstones).
3. Read [`AGENTS.md`](./AGENTS.md) for cross-vendor agent rules and
   [`CONSTITUTION.md`](./CONSTITUTION.md) for the non-negotiables.
4. Open [`docs/INDEX.md`](./docs/INDEX.md) and pick the doc relevant
   to your task — do not load the whole repo.
5. Use `cvg pr stack` before merging — it surfaces the conflict matrix.
