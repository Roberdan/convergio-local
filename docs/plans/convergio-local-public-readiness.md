---
type: Plan
status: Active
owner: Convergio
updated: 2026-04-29
source_of_truth: repo
---

# Convergio Local public readiness plan

## Objective

Prepare Convergio Local for a public `v0.1.0` release as a local-first,
SQLite-only control plane for safe parallel AI agents.

The release must prove:

- local setup works;
- multiple agents can share one daemon safely;
- task/evidence/audit/gates are reliable;
- CRDT storage foundations exist before sync is needed;
- workspace coordination prevents unsafe parallel file/Git work;
- future capabilities can be added without turning the repo into a
  monolith.

## Current state

Implemented:

| Area | State |
|------|-------|
| Local daemon | SQLite-only, localhost default |
| CLI | setup, doctor, demo, task/evidence flow, service management |
| MCP | `convergio.help` and `convergio.act` |
| Agent actions | plan/task/evidence/audit/refusal actions |
| Audit | hash-chained verification |
| Gates | evidence, no-debt, no-stub, no-secrets, zero-warnings |
| Release | local package script, macOS signing/notarization docs |
| Docs | vision, multi-agent model, CRDT/workspace/capability/ACP ADRs |
| Context hygiene | folder-local `AGENTS.md` and `CLAUDE.md` for crates/docs |

Not implemented:

| Area | Needed before public v0.1 |
|------|---------------------------|
| CRDT | schema, merge engine, conflict UX, E2E |
| Workspace | resources, leases, patch proposals, merge arbiter, E2E |
| Agent context | task context packets and bus actions |
| Capabilities | signature-first registry/install/disable model |
| Supply chain | audit/deny/SBOM/provenance |
| Public repo | final `convergio-local` repo/release setup |

## Invariants

| ID | Rule |
|----|------|
| inv-local | SQLite-only local runtime for v0.1 |
| inv-daemon | agents coordinate through daemon APIs, not direct SQLite |
| inv-audit | visible state changes must be auditable |
| inv-gates | clients cannot bypass server-side gates |
| inv-context | every crate has local `AGENTS.md` and `CLAUDE.md` |
| inv-plans | durable plans live under `docs/plans/` |
| inv-capabilities | remote capability install requires signature verification |
| inv-workspace | agents do not directly mutate canonical workspace once leases exist |

## Phase order

| Phase | Goal | Depends on |
|-------|------|------------|
| P0 | repo/documentation cleanup | none |
| P1 | CRDT storage foundation | P0 |
| P2 | workspace coordination foundation | P1 |
| P3 | multi-agent context/bus actions | P1 |
| P4 | runner adapter proof | P2, P3 |
| P5 | capability manager foundation | P0 |
| P6 | supply-chain/public release | P1, P2, P5 |
| P7 | ACP read-only proof | P3 |

## Task graph

| Task ID | Phase | Depends on | Output | Acceptance |
|---------|-------|------------|--------|------------|
| crdt-core-schema | P1 | none | migration + store types for actors/ops/cells/clocks | schema migrates; no existing tests regress |
| crdt-merge-engine | P1 | crdt-core-schema | deterministic merge helpers | two-actor unit tests pass |
| crdt-conflict-ux | P1 | crdt-merge-engine | API/CLI/MCP conflict reporting | unresolved conflict is visible and blocks unsafe completion |
| crdt-e2e-tests | P1 | crdt-conflict-ux | cross-layer CRDT E2E | audit verifies after imported ops |
| workspace-resource-model | P2 | crdt-core-schema | resources, leases, sessions, conflicts schema/store | agents can claim/release file resources |
| patch-proposal-flow | P2 | workspace-resource-model | patch proposal API and validation | stale base/path escape/same-file conflict refused |
| merge-arbiter | P2 | patch-proposal-flow | serialized apply/test/audit loop | accepted patches reach canonical workspace only through arbiter |
| workspace-e2e-tests | P2 | merge-arbiter | multi-agent workspace tests | two agents same-file conflict; different files merge |
| agent-registry | P3 | crdt-core-schema | explicit agent sessions/roles/skills | each worker has stable identity and heartbeat |
| context-packets | P3 | agent-registry | compact task context generator | worker prompt excludes unrelated repo history |
| bus-mcp-actions | P3 | context-packets | message publish/poll/ack via `convergio.act` | agents coordinate through plan-scoped bus |
| runner-adapter-proof | P4 | workspace-e2e-tests, bus-mcp-actions | one real local runner adapter | Convergio can launch one worker kind safely |
| capability-registry-core | P5 | none | installed capability registry | local registry persists installed/disabled state |
| capability-signatures | P5 | capability-registry-core | signed package verification | bad/unsigned package refused |
| local-capability-install | P5 | capability-registry-core, capability-signatures | local package install/disable | staging + atomic install works |
| capability-uninstall-rollback | P5 | local-capability-install | disable/remove/rollback semantics | failed migration/install rolls back |
| planner-capability | P5 | local-capability-install | planner extracted or wrapped as first capability | `planner.solve` action works through `convergio.act` |
| supply-chain-ci | P6 | none | cargo deny/audit/SBOM/provenance | release artifacts have policy checks and attestations |
| remote-capability-registry | P6 | local-capability-install, capability-signatures, supply-chain-ci | first-party remote registry | remote install only after signature verification |
| public-v010-release | P6 | crdt-e2e-tests, workspace-e2e-tests, supply-chain-ci, planner-capability | public repo + signed release | public install path documented and verified |
| acp-readonly-poc | P7 | bus-mcp-actions | read-only ACP bridge proof | editor can read Convergio status without bypassing gates |

## Ready queue

Only tasks with no unmet dependencies are safe to start in parallel.

| Task ID | Scope | Why ready |
|---------|-------|-----------|
| crdt-core-schema | `crates/convergio-durability`, migrations, tests | foundation for CRDT, workspace, and agent registry |
| supply-chain-ci | CI/release/dependency policy files | independent of runtime schema work |

Do not start workspace, runner, public release, ACP, or capability install
tasks until their dependencies in the task graph are complete.

## Acceptance criteria

Public `v0.1.0` is allowed only when:

1. `cargo fmt --all -- --check` passes.
2. `RUSTFLAGS="-Dwarnings" cargo clippy --workspace --all-targets -- -D warnings` passes.
3. `RUSTFLAGS="-Dwarnings" cargo test --workspace` passes.
4. CRDT E2E proves deterministic two-actor merge and conflict surfacing.
5. Workspace E2E proves same-file conflict and safe different-file merge.
6. MCP agent flow can claim, heartbeat, add evidence, submit, explain
   refusal, and use bus actions.
7. Release packaging is reproducible enough for local verification.
8. macOS artifact signing/notarization path is documented and tested.
9. Public README does not claim future behavior as shipped.
10. Every crate has `AGENTS.md` and `CLAUDE.md`.

## Validation commands

```bash
cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" cargo clippy --workspace --all-targets -- -D warnings
RUSTFLAGS="-Dwarnings" cargo test --workspace
sh scripts/package-local.sh
cvg doctor --json
cvg demo
```

## Links

| File | Purpose |
|------|---------|
| `docs/vision.md` | product direction |
| `docs/multi-agent-operating-model.md` | swarm operating model |
| `docs/adr/0006-crdt-storage.md` | CRDT decision |
| `docs/adr/0007-workspace-coordination.md` | workspace decision |
| `docs/adr/0008-downloadable-capabilities.md` | capability decision |
| `docs/adr/0009-agent-client-protocol-adapter.md` | ACP decision |
| `docs/agent-instruction-guidelines.md` | agent Markdown/prompt rules |
| `CONSTITUTION.md` | non-negotiable repo rules |

## Next executable step

Start P1 with `crdt-core-schema`.

Required first implementation slice:

1. add migration for `crdt_actors`, `crdt_ops`, `crdt_cells`,
   `crdt_row_clocks`;
2. add typed store module in `convergio-durability`;
3. add local actor creation/load;
4. add idempotent op append;
5. add tests for actor creation and duplicate op import;
6. keep existing materialized tables intact.
