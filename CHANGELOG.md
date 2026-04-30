# Changelog

All notable changes to Convergio will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows Semantic Versioning before 1.0 with explicit
MVP scope notes.

## [0.1.2](https://github.com/Roberdan/convergio-local/compare/convergio-local-v0.1.1...convergio-local-v0.1.2) (2026-04-30)


### Bug Fixes

* **ci:** run release workflow for component tags ([313c8ff](https://github.com/Roberdan/convergio-local/commit/313c8ff6312fb694e9d4c2fb2ed3784ccc7b4825))

## [0.1.1](https://github.com/Roberdan/convergio-local/compare/convergio-local-v0.1.0...convergio-local-v0.1.1) (2026-04-30)


### Features

* **api:** add agent action contract ([2aacd08](https://github.com/Roberdan/convergio-local/commit/2aacd080ba9e31933f4f824316fc748bbbeb703c))
* **bus,server:** implement Layer 2 agent message bus ([3426b38](https://github.com/Roberdan/convergio-local/commit/3426b38be4d819ed8ad5dea542da02e9b5da6ee3))
* **capability:** add disable and remove flow ([fedbc27](https://github.com/Roberdan/convergio-local/commit/fedbc27f7ff5d63d20bdc8a37c0d680244830a79))
* **cli:** add local demo and task workflow ([5936a68](https://github.com/Roberdan/convergio-local/commit/5936a6814d326bf467773ea708432e92c34e0db7))
* **cli:** add local setup and doctor ([85332ea](https://github.com/Roberdan/convergio-local/commit/85332ea5eb66de2e477c0da47ef0d4ec4d35c01a))
* **cli:** add local status dashboard ([2c8e728](https://github.com/Roberdan/convergio-local/commit/2c8e7282efb5383ab7f992e01525728c60d04e15))
* **cli:** add productization workflow ([b67ac6d](https://github.com/Roberdan/convergio-local/commit/b67ac6d7688066927c82336076c3361d23d127c1))
* **durability,docs:** three sacred principles + multilingua NoDebtGate + ZeroWarningsGate ([6c9e7dd](https://github.com/Roberdan/convergio-local/commit/6c9e7dd76eb0a7d8c16017bcc28ce6add25ee9d8))
* **durability,server:** add Layer 1 reaper loop ([95d608b](https://github.com/Roberdan/convergio-local/commit/95d608b3ccb10239d9c2578f737dc6be49761404))
* **durability:** add capability registry core ([317176d](https://github.com/Roberdan/convergio-local/commit/317176daf525f29dfb6daf44ec6e58da689d636a))
* **durability:** add CRDT core storage ([9bd3819](https://github.com/Roberdan/convergio-local/commit/9bd38199e5cbe455aded79b1e1a0d93b609bbafb))
* **durability:** add durable agent registry ([d34c631](https://github.com/Roberdan/convergio-local/commit/d34c63130aa5a74defec1ddec2a4de5c34e08fb3))
* **durability:** add workspace resource leases ([ba473a8](https://github.com/Roberdan/convergio-local/commit/ba473a826dcc4c007957b07220fe4283ea701a6c))
* **durability:** arbitrate workspace merge queue ([c5cbad8](https://github.com/Roberdan/convergio-local/commit/c5cbad859870083e1793a97a63298a4507219b23))
* **durability:** audit CRDT imports ([4963067](https://github.com/Roberdan/convergio-local/commit/4963067ea7d533420883a83abea8f1cae60aefcd))
* **durability:** block unresolved CRDT conflicts ([0b87c7f](https://github.com/Roberdan/convergio-local/commit/0b87c7f7766eeea3aea3a6324c50e84fc38b1099))
* **durability:** materialize CRDT cells ([49c2924](https://github.com/Roberdan/convergio-local/commit/49c2924d57b0179eff50689f8ab4f5a43b1b9b02))
* **durability:** NoDebtGate — refuse evidence with debt markers ([7c7ab9f](https://github.com/Roberdan/convergio-local/commit/7c7ab9f680c910e787783e0b98070662effe3ca5))
* **durability:** P4 NoStubGate — refuse scaffolding-only evidence ([02fb217](https://github.com/Roberdan/convergio-local/commit/02fb2174f018e123b678bc6b544d0fb19a817fd7))
* **durability:** validate workspace patch proposals ([e8e8ce0](https://github.com/Roberdan/convergio-local/commit/e8e8ce0759068e197c461ad03717a77b9262dfcd))
* **durability:** verify capability signatures ([638c2b0](https://github.com/Roberdan/convergio-local/commit/638c2b0242ba426d69b1860b5121058dfeccb78f))
* **i18n,cli,docs:** P5 internationalization first — Italian + English day one ([66a310b](https://github.com/Roberdan/convergio-local/commit/66a310b0519be9a92d2890d62ae122504ab60fbc))
* **lifecycle,planner,thor,executor,server,cli:** Layer 3 watcher + Layer 4 ([11e21a9](https://github.com/Roberdan/convergio-local/commit/11e21a9d0536efcb97a19056f751e4ac67fd2435))
* **lifecycle,server:** implement Layer 3 supervisor + HTTP surface ([01e289d](https://github.com/Roberdan/convergio-local/commit/01e289de931c9c8f615f742abd35a0e9a4bad238))
* **lifecycle,server:** Layer 3 OS-watcher loop ([9deecb2](https://github.com/Roberdan/convergio-local/commit/9deecb2208511cae9bc4e8dd0ef127facd0ad31b))
* **mcp:** add local agent bridge ([6b6fc2b](https://github.com/Roberdan/convergio-local/commit/6b6fc2b6cb8f7f8a2975fc65b6dd6346892475c5))
* **mcp:** expose plan bus actions ([a2ab720](https://github.com/Roberdan/convergio-local/commit/a2ab7205e7d86281cb4ca7e12a9b61330bae003d))
* **planner:** expose planner capability action ([647f895](https://github.com/Roberdan/convergio-local/commit/647f89543977786b197d0fbedf7c969ab3ae4d9c))
* **server,cli:** wire HTTP layer + cvg CLI + end-to-end test ([13c829f](https://github.com/Roberdan/convergio-local/commit/13c829f04fb007133decba18df4615848fc0c772))
* **server:** add task context packets ([89d5688](https://github.com/Roberdan/convergio-local/commit/89d56881b236f2a3bad4c706f3c874363ccf04e0))
* **server:** install signed capability packages ([6c84515](https://github.com/Roberdan/convergio-local/commit/6c84515749e0bbf41875c9769349d6c24c3e82c3))
* **server:** prove local shell runner ([ecfae30](https://github.com/Roberdan/convergio-local/commit/ecfae30633d86a9e7ffdf85c2fa4866b62252baa))


### Bug Fixes

* **ci:** align public release checks ([dd5a98e](https://github.com/Roberdan/convergio-local/commit/dd5a98e8ac6cd792a9a583fa99a0eefcdb9ffac5))
* **ci:** trigger lockfile sync for workspace manifest ([c4ec10f](https://github.com/Roberdan/convergio-local/commit/c4ec10f8a6095edaf5f0727b237037d6faa31cb8))
* **cli:** keep doctor JSON stderr clean ([c0500b2](https://github.com/Roberdan/convergio-local/commit/c0500b2ab31e63be7a931b27b12fccaad88087eb))
* **db:** wait for sqlite write locks ([e9b9dcb](https://github.com/Roberdan/convergio-local/commit/e9b9dcbae0705264583d3c964c438f8f4b30dacf))
* **durability:** harden local audit and gates ([66006e3](https://github.com/Roberdan/convergio-local/commit/66006e3092d956bdd5e2677714432cf65f148d00))
* **repo:** replace shadowed binaries atomically ([0c1472f](https://github.com/Roberdan/convergio-local/commit/0c1472f3a90f3e41d2c6abb3423d70173ec6c4e3))


### Refactoring

* **repo:** focus runtime on local SQLite ([4e025a6](https://github.com/Roberdan/convergio-local/commit/4e025a6642e1b5e195642f760706fbe9c4192c58))


### Documentation

* **plans:** clarify execution dependencies ([bebd249](https://github.com/Roberdan/convergio-local/commit/bebd24983df1c526343345f094ae3030308f03e0))
* **plans:** define public push sequence ([1c99b66](https://github.com/Roberdan/convergio-local/commit/1c99b662f67f1113b59d49cbcc4b58fb1c30a528))
* **plans:** record public push validation ([a97874b](https://github.com/Roberdan/convergio-local/commit/a97874bb77e3bf70800d5c0bd6ff3678fe16ced7))
* **plans:** sync public readiness queue ([90c81a5](https://github.com/Roberdan/convergio-local/commit/90c81a51f05341cb2eda19c6ee0b07d16d4498a2))
* **release:** align v0.1 public docs ([85b79ce](https://github.com/Roberdan/convergio-local/commit/85b79ce9b6bacf59be578c38437cd45f5a7799ff))
* **release:** document macos notarization flow ([0d3dde7](https://github.com/Roberdan/convergio-local/commit/0d3dde7e4f89ea2368ac7efd2ef6b1002dcd3f1d))
* **release:** record public publication ([a1bef7c](https://github.com/Roberdan/convergio-local/commit/a1bef7c318ae06c20681f79f6f5ff53aaf904eb2))
* **release:** record v0.1 validation ([7f3c380](https://github.com/Roberdan/convergio-local/commit/7f3c380abdfbf2e6d608c41c5c06702e44982408))
* **release:** refresh notarized artifact metadata ([3588137](https://github.com/Roberdan/convergio-local/commit/3588137a1a80e19306e58b877c9497e92b23c9f9))
* **repo,server:** refresh CHANGELOG, ROADMAP, server README, status ([558234d](https://github.com/Roberdan/convergio-local/commit/558234d047f440f302b40bc9bfeec91b9487c6b9))
* **repo:** align public readiness claims ([9d30701](https://github.com/Roberdan/convergio-local/commit/9d30701fae1c4f75bca109029dfb826e6e0082a3))
* **repo:** codify multi-agent governance ([09729e4](https://github.com/Roberdan/convergio-local/commit/09729e4a8f2194ddb2ca6f9195dd5b10ea88f5c6))

## [Unreleased]

No unreleased changes.

## [0.1.0] - 2026-04-30

### Added

- Initial Convergio Local workspace, with layered Rust crates for DB,
  durability, bus, lifecycle, server, CLI, planner, validator and executor.
- SQLite-backed local daemon, localhost HTTP API, pure HTTP `cvg` CLI and
  one-command local install flow.
- Layer 1 durability: plans, tasks, evidence, gates, reaper and
  hash-chained audit verification.
- Layer 2 bus: persistent local plan-scoped messages with publish, poll and
  ack actions.
- Layer 3 lifecycle: local process spawn, heartbeat and watcher.
- Layer 4 reference flow: planner, executor tick, Thor validator and
  `planner.solve` capability-gated action.
- Server-side gate pipeline, including evidence, wave sequencing, no-debt,
  no-stub, no-secrets and zero-warning gates.
- Guided `cvg demo`, local task/evidence commands, service management,
  setup, doctor diagnostics, MCP logs and `cvg mcp tail`.
- Shared typed agent action contract and stdio MCP bridge with
  `convergio.help` and `convergio.act`.
- CRDT storage foundation for multi-actor row/column state.
- Workspace coordination foundation: resources, leases, patch proposals,
  merge queue arbitration and conflict reporting.
- Durable agent registry, task context packets and plan-scoped bus actions
  for multi-agent coordination through the daemon.
- Local capability registry, Ed25519 signature verification, signed local
  `install-file`, disable and remove safety.
- Constrained local shell runner proof through `spawn_runner`.
- English and Italian Fluent bundles with coverage tests.
- Release artifact workflow, local packaging script, macOS signing and
  notarization documentation.
- Project docs: README, Architecture, Constitution, Roadmap, Security,
  Contributing, Code of Conduct, ADRs and public readiness plan.
- Convergio Community License v1.3 (source-available, aligned with the
  legacy `github.com/Roberdan/convergio` repo).

### Changed

- Repositioned the project as a **single-user, local-first, SQLite-only**
  runtime.
- Removed remote deployment and account-model language from current
  documentation.
- Removed the legacy plan scope field from the plan model, schema, API
  and CLI.
- Added a minimal `convergio start` command parser so `convergio --help`
  works and the documented quickstart is real.
- Removed the unused scaffold-only worktree crate from the workspace.
- Updated README, Architecture, Constitution, Security, Roadmap, ADR
  references and crate READMEs around the focused local MVP.
