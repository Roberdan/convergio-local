---
id: 0032
status: accepted
date: 2026-05-03
topics: [layer-3, layer-4, runners, agents, cost, auth]
related_adrs: [0011, 0014, 0027, 0028, 0029]
touches_crates: [convergio-runner, convergio-executor, convergio-cli]
last_validated: 2026-05-03
---

# 0032. Vendor-CLI runners (no raw API calls)

- Status: accepted
- Date: 2026-05-03
- Tags: agents, runners, cost, auth

## Context

Convergio's executor (Layer 4) needs to spawn agent sessions for
pending tasks. The MVP `SpawnTemplate::default()` shells out to
`/bin/echo` — placeholder. The next step is to spawn real LLM
agents.

There are two ways to reach an LLM agent:

1. The vendor's CLI (`claude` from Anthropic, `copilot` from
   GitHub) — interactive by default, also supports `-p` /
   `--prompt` for non-interactive single-shot runs with model
   selection (`--model sonnet|opus|gpt-5.2|...`) and built-in
   permission prompts.
2. The vendor's HTTP API directly (Anthropic Messages API,
   OpenAI Chat Completions). Requires an API key.

## Decision

**Always go through the CLI. Never call the raw HTTP API.**

The runner crate (`convergio-runner`) ships two implementations:

- `ClaudeRunner` invokes `claude -p --model <X> --output-format
  json --input-format text` and pipes the prompt on stdin.
- `CopilotRunner` invokes `copilot -p <prompt> --model <X>
  --allow-all-tools` (Copilot requires the all-tools flag for any
  non-interactive run; Convergio's worktree boundary + audit chain
  are the actual safety net).

The wire format `RunnerKind = "<vendor>:<model>"` is what gets
stored in `agents.kind` and what `cvg agent spawn --runner`
accepts. New models surface immediately — the runner forwards the
model string to the CLI without a Convergio release.

## Consequences

### Why this is the right call

- **Cost lives where the operator already sees it.** A Claude Max
  plan and a Copilot subscription both bill the operator directly
  via the vendor; running the CLI consumes that subscription, never
  re-bills via API. No double-charging risk.
- **Auth lives where the operator already configured it.** No API
  keys in env vars, no key rotation in Convergio config, no
  attack surface for key exfiltration. The CLI's existing OAuth
  / token store is the boundary.
- **Permission policy is vendor-curated.** Both CLIs have a
  consent model (`--allow-tool`, `--deny-url`, ...) and an audit
  log of their own. Convergio adds the worktree sandbox + the
  daemon's audit chain on top.
- **Observability survives.** The CLI prints structured JSON on
  `--output-format json`, which the executor captures verbatim
  into evidence rows.
- **Replaceable.** Whatever ship `claude` ships next, the
  Convergio side moves zero code — the runner only knows the argv
  contract.

### Trade-offs accepted

- Single-machine binding: the runner only works where the CLIs
  are installed and authenticated. Convergio is a local-first
  daemon by design (CONSTITUTION § Sacred principle 4 — fully
  wired) so this is the same scope.
- Cold-start latency: spinning a CLI subprocess is ~200ms slower
  than reusing an HTTP connection. Negligible against the seconds
  spent in the model itself.
- Some vendor flags differ between releases (e.g. Copilot's
  `--allow-all-tools` is current at the time of writing). Each
  runner pins its argv contract; bumping a vendor major version
  is a one-file edit per runner.

## Alternatives rejected

- **Direct Anthropic / OpenAI API calls** — would split auth and
  billing across two systems and require API key management.
  Rejected.
- **A single `--runner-cmd` template with arbitrary argv** — too
  generic; the prompt contract differs per vendor (Copilot wants
  `--allow-all-tools`, Claude wants `--input-format text`), and
  the right place to encode that is per-vendor code.
- **An MCP-only path** — `convergio-mcp` already exposes the
  daemon to Claude Code as MCP server. That's the *agent ↔
  daemon* protocol; it does not solve "how does the daemon
  spawn an agent in the first place". The two layers compose.

## Where it lives

- Crate `crates/convergio-runner/` — the trait + the two impls +
  the prompt builder.
- The executor (`convergio-executor`) will read a per-task
  runner kind (default: `claude:sonnet`) and dispatch via
  `runner::for_kind(&kind).prepare(&ctx)`. That wiring ships in a
  follow-up PR — this ADR documents the boundary, not the wiring.
- `cvg agent spawn --task <id> --runner <kind>` is the manual
  one-shot surface; same follow-up.

## Validation gates

- `RUSTFLAGS=-Dwarnings cargo clippy -p convergio-runner --all-targets -- -D warnings`
- `cargo test -p convergio-runner` — argv shape + prompt
  composition tests (no subprocess execution).
- Manual smoke (operator-side): `cvg agent spawn --task <id>
  --runner claude:sonnet --dry-run` prints the argv + prompt
  without execution. The same minus `--dry-run` runs the real
  CLI against the operator's existing auth.
