# Roadmap

This is the 8-week MVP plan from
[docs/spec/v3-durability-layer.md](./docs/spec/v3-durability-layer.md),
operationalized.

Mark items as `[x]` when shipped. Move items between weeks freely.

---

## Week 1-2 — Cleanup + Layer 1 hardening

- [x] Workspace skeleton with the 10 KEEP crates
- [x] `convergio-db` abstraction with SQLite feature
- [ ] `convergio-db` Postgres feature behind `--features postgres`
- [x] `convergio-durability`: `plans`, `tasks`, `evidence`, `agents`, `audit_log` schema
- [x] Hash-chained audit log + `GET /v1/audit/verify`
- [x] Gate pipeline (`PlanStatusGate`, `EvidenceGate`, `WaveSequenceGate`)
- [x] HTTP surface (`/v1/plans`, `/v1/tasks`, `/v1/audit/verify`, `/v1/health`)
- [x] `cvg` CLI (pure HTTP client) with `health`, `plan`, `audit` subcommands
- [x] End-to-end test booting the router in-process and exercising the gate pipeline
- [x] Local `fmt + clippy -D warnings + test` clean
- [ ] Reaper loop (60s) — releases stale tasks
- [ ] CI workflow green on first push (already authored in `.github/workflows/ci.yml`)

## Week 3-4 — Layer 2 + Layer 3

- [x] `convergio-bus`: `agent_messages` table, publish + poll + ack
- [x] HTTP surface: `POST /v1/plans/:id/messages`, `GET` with cursor, `POST /v1/messages/:id/ack`
- [x] Per-plan FIFO + at-least-once delivery + persistent
- [ ] Direct messaging convention (`topic = "agent:<id>"`) — works today, not yet documented as first-class
- [x] `convergio-lifecycle`: `agent_processes` table
- [x] `POST /v1/agents/spawn` launches a process and tracks it
- [x] Heartbeat endpoint + Supervisor::mark_exited
- [ ] OS-watcher loop that flips status to `exited`/`failed` on real child exit
- [ ] E2E test: spawn agent → kill it → reaper notices → task re-queued

## Week 5-6 — Layer 4 minimal viable

- [ ] `cvg solve "<mission>"` produces a plan in DB (LLM-free heuristic + optional LLM)
- [ ] `cvg start <plan_id>` runs the executor loop
- [ ] Thor validator process — separate Layer 3 spawn
- [ ] Worktree integration (`convergio-worktree`)
- [ ] **Quickstart smoke**: `convergio start && cvg solve "build me a todo CLI" && cvg start <id>` produces a real PR-ready output in 5 minutes

## Week 7 — README, landing, demo

- [ ] 7-word value prop tested on 5 friends
- [ ] 60-second demo video: gate rejection + heartbeat reaper + audit chain verify
- [ ] Landing page (single page, no architecture diagram)
- [ ] Re-pin README — quickstart first, no wall-of-architecture

## Week 8 — Outreach

- [ ] Message Antonio Gatti (healthcare design partner candidate)
- [ ] HN Show launch
- [ ] 5-10 direct outreach to healthcare / compliance circles
- [ ] 3 PRs against repos that could use this (Claude Code skill authors wanting audit trail, etc.)

---

## Success criteria (do not move)

- 3 external adopters running their workflows on top of Convergio
- 10 buyer conversations in healthcare / regulated AI
- 0 new features outside this roadmap

## Deferred (post-MVP)

`convergio-mesh` (multi-host), `convergio-doctor` (integration test bundle),
slim `convergio-mcp-server`, knowledge / catalog, kernel / MLX integration,
night agents, skills-on-demand, billing.
