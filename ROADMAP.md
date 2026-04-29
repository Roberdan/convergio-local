# Roadmap

Focused MVP: **single-user, local-first, SQLite-only Convergio**.

The goal is not to become a hosted platform. The goal is to solve one
concrete problem well: local AI agents should not be able to claim
"done" without durable state, evidence, auditability and server-side
quality gates.

## Shipped foundation

- [x] SQLite-backed workspace with local daemon and `cvg` CLI
- [x] Plans, tasks, evidence and audit log
- [x] Hash-chained audit verification (`GET /v1/audit/verify`)
- [x] Gate pipeline with evidence, wave sequence, no-debt, no-stub and
      zero-warning gates
- [x] Persistent local agent message bus
- [x] Agent process spawn, heartbeat and watcher loop
- [x] Reaper loop for stale task recovery
- [x] Reference planner, executor tick and Thor validator
- [x] Guided `cvg demo` showing gate refusal, clean validation and audit
      verification
- [x] Local task/evidence CLI commands for the core manual loop
- [x] One-command local install script
- [x] Non-local daemon bind requires explicit opt-in
- [x] English/Italian i18n crate and CLI `--lang`
- [x] HTTP E2E tests for the local runtime

## Next focus

- [ ] Make audit writes atomic with the corresponding state changes
- [ ] Make audit and message sequence allocation robust under concurrent
      writes
- [ ] Reap tasks that were dispatched but never heartbeat
- [ ] Add a first `NoSecretsGate` for common local evidence leaks
- [ ] Add CLI output modes (`human`, `json`, `plain`) for accessibility
- [ ] Replace the deterministic reference executor with a practical local
      adapter for one real agent runner
- [ ] Add packaged release artifacts beyond `cargo install --path`

## Explicitly out of scope

- hosted service
- remote multi-user deployment
- account or organization model
- RBAC
- distributed mesh
- graphical UI
- billing
- agent marketplace

## Success criteria

- A solo developer can install the daemon and CLI, run the quickstart,
  and see a gate refusal plus audit verification in minutes.
- The local daemon remains easy to explain: one process, one SQLite
  file, localhost HTTP, evidence gates.
- No new feature expands the product beyond the local-first scope unless
  real users prove the need.
