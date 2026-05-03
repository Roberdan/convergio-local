---
id: 0033
status: accepted
date: 2026-05-03
topics: [runners, security, agents, permissions]
related_adrs: [0032]
touches_crates: [convergio-runner, convergio-cli]
last_validated: 2026-05-03
---

# 0033. Vendor-CLI runners use least-privilege permission profiles

- Status: accepted
- Date: 2026-05-03
- Tags: security, permissions, runners

## Context

ADR-0032 made vendor CLIs (`claude`, `copilot`) the only sanctioned
spawn surface for Convergio agents. The first cut, however, used
the vendor's nuke flags to make non-interactive runs work at all:

- Claude — `--dangerously-skip-permissions` (skips every tool
  consent check).
- Copilot — `--allow-all-tools` (same, in their vocabulary).

The first end-to-end smoke (PR #124, `claude:sonnet` implementing
F26) confirmed those flags were necessary *only* because the
agents need permission-free tool access in non-interactive mode.
But it also highlighted the contradiction: Convergio is the
*leash*. Granting a spawned agent the entire process's privileges
is the opposite of leashing it.

Both CLIs ship a granular permission API:

- Claude: `--permission-mode`, `--allowed-tools "Bash(git *) Edit"`.
- Copilot: `--allow-tool='shell(git:*)'`,
  `--deny-tool='shell(git push)'`, `--add-dir`,
  `--allow-url`, `--deny-url`.

## Decision

Replace the nuke flags with named **permission profiles** that
each runner translates into vendor-specific allow/deny lists:

- **`Standard`** (default) — code-implementing tasks: build, edit
  files in the worktree, talk to the daemon (`cvg`), open PRs via
  `gh`. Forbids `rm`, `sudo`, `git push origin main`,
  `git push --force`, `git reset --hard`, `chmod 777`.
- **`ReadOnly`** — only Read / Glob / Grep / TodoWrite. No Bash,
  no Edit. For triage and inspection tasks.
- **`Sandbox`** — keep the legacy nuke flags
  (`--dangerously-skip-permissions` / `--allow-all`). Reserved for
  sealed VMs where the audit chain plus the worktree boundary are
  already isolated by the host. Never the operator's main
  checkout.

The wire format:

```text
cvg agent spawn --task <id> --runner claude:sonnet --profile standard
cvg agent spawn --task <id> --runner copilot:gpt-5.2 --profile read_only
cvg agent spawn --task <id> --runner claude:opus --profile sandbox
```

`--profile` defaults to `standard`. Operators wanting `Sandbox`
must pass it explicitly, and the help text says so.

The Copilot deny list is **always** applied, even on `Sandbox`,
because the audit chain forbids those commands forever (no
operator should be able to opt back into `rm -rf /` from inside
Convergio).

## Consequences

- The first sacred principle (Constitution P1) holds: an agent
  spawned by Convergio cannot run `rm -rf` or push to `main`,
  even if it is hallucinating.
- The agent contract in `prompt::build` is unchanged — the
  prompt still tells the agent *what* to do; the profile
  controls *what it can do*.
- New profiles are a one-line addition to the enum + matching
  arms in two methods (`claude_allowed_tools`,
  `copilot_allow_tools`).
- `Sandbox` exists as an explicit escape hatch so test rigs +
  agent-sandbox VMs that *are* isolated can still ship without a
  separate code path.
- The whitelist is necessarily slightly behind reality (e.g. if
  a new `cvg` subcommand needs a tool not yet in the list, the
  agent will surface it as a permission denial). That is the
  desired direction — the operator notices and decides.

## Alternatives considered

- **Keep `--dangerously-skip-permissions` as default** — what we
  shipped first. Rejected once the meaning sank in.
- **One bespoke profile per task type** — over-engineered. Three
  profiles cover 99% of the cases; the rest can use `Sandbox`
  with a code reviewer's eye.
- **Server-side permission policy** (the daemon vetoes specific
  tool calls) — would require routing every tool call through
  the daemon, breaking the simple subprocess contract. Out of
  scope.

## Validation

- `cargo test -p convergio-runner` — 18 unit + 6 integration
  tests, including:
  - `Standard` profile MUST NOT include `--dangerously-skip-permissions`.
  - `Standard` profile MUST include `--add-dir <worktree>`.
  - Whitelist MUST include `shell(cargo:*)`, `shell(cvg:*)`.
  - Deny list MUST include `shell(rm:*)`, `sudo`, `push origin main`.
  - `Sandbox` profile MUST keep the nuke flags (escape-hatch
    contract).
- Manual smoke against the running daemon: `cvg agent spawn
  --dry-run` prints argv with `--permission-mode acceptEdits`
  + `--allowed-tools "..."` for Claude, or
  `--allow-tool '...' --deny-tool '...' --add-dir <wt>` for
  Copilot.
