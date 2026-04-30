# Roadmap

Focused MVP: **single-user, local-first, SQLite-only Convergio**.

The goal is not to become a hosted platform. The goal is to solve one
concrete problem well: local AI agents should be able to work in
parallel without corrupting durable state, files, Git history or CI, and
without claiming "done" before evidence, gates and audit accept the work.

## Shipped foundation

- [x] SQLite-backed workspace with local daemon and `cvg` CLI
- [x] Plans, tasks, evidence and audit log
- [x] Hash-chained audit verification (`GET /v1/audit/verify`)
- [x] Atomic state + audit writes for facade/reaper operations
- [x] Robust local sequence allocation for audit and bus writes
- [x] Gate pipeline with evidence, wave sequence, no-debt, no-stub and
      zero-warning gates
- [x] `NoSecretsGate` for common credential leaks in evidence
- [x] Persistent local agent message bus
- [x] Agent process spawn, heartbeat and watcher loop
- [x] Reaper loop for stale and never-heartbeated task recovery
- [x] Reference planner, executor tick and Thor validator
- [x] Guided `cvg demo` showing gate refusal, clean validation and audit
      verification
- [x] Local task/evidence CLI commands for the core manual loop
- [x] One-command local install script
- [x] Local setup and doctor diagnostics
- [x] Shared agent action contract for MCP/adapters
- [x] Minimal stdio MCP bridge with `convergio.help` and `convergio.act`
- [x] Full typed MCP action coverage behind `convergio.act`
- [x] Generated adapter snippets via `cvg setup agent <host>`
- [x] MCP action log and `cvg mcp tail`
- [x] User-level service management via `cvg service`
- [x] Release artifact workflow and local packaging script
- [x] Durable gate refusal explanation through audit
- [x] Global `--output human|json|plain` foundation for health/doctor
- [x] Non-local daemon bind requires explicit opt-in
- [x] English/Italian i18n crate and CLI `--lang`
- [x] HTTP E2E tests for the local runtime
- [x] Multi-agent operating model documented
- [x] Folder-local agent guardrails for crates/docs
- [x] CRDT storage foundation for multi-actor row/column state
- [x] Workspace coordination foundation: resources, leases, patch
      proposals, merge queue and conflicts
- [x] Task context packets and plan-scoped bus actions in MCP
- [x] Local capability registry and Ed25519 signature verification
- [x] Signed local capability `install-file`
- [x] Capability disable/remove safety
- [x] Local shell runner adapter proof
- [x] Planner capability action (`planner.solve`)

## Next focus

- [ ] Extend `--output human|json|plain` beyond health/doctor to every CLI command
- [ ] Replace the deterministic reference executor with product adapters
      for real hosted/local agent tools beyond the shell proof
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
- Multiple local agents can work in parallel without directly mutating
  the canonical workspace or silently overwriting each other's state.
- No new feature expands the product beyond the local-first scope unless
  real users prove the need.
