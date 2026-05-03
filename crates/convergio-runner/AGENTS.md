# AGENTS.md — convergio-runner

For repo-wide rules see [../../AGENTS.md](../../AGENTS.md). For
the rationale see
[../../docs/adr/0032-vendor-cli-runners.md](../../docs/adr/0032-vendor-cli-runners.md).

This crate is the **runner layer**: it turns a Convergio task into a
prepared subprocess command that drives the operator's already-paid,
already-authenticated vendor CLI (`claude`, `copilot`).

## Invariants

- **No raw Anthropic / OpenAI HTTP calls.** Ever. ADR-0032. The
  vendor CLI is the only sanctioned cost + auth surface.
- **Pure preparation.** `Runner::prepare` builds a `PreparedCommand`;
  it does not spawn, does not network, does not write files. The
  executor / `cvg agent spawn` materialises a real
  `std::process::Command` and runs it.
- **One prompt shape, two vendors.** `prompt::build` is the single
  source of truth for the agent contract (heartbeat, evidence,
  transitions, PR conventions). New vendors must reuse it.
- **Long prompts via stdin where supported.** `claude -p` reads off
  stdin (`--input-format text`), so the graph context-pack can be
  arbitrarily large without hitting argv limits. Copilot today only
  takes the prompt on argv — when that lifts, switch to stdin
  symmetrically.
- **`RunnerKind` round-trips through `Display`/`FromStr`.** The wire
  format `<vendor>:<model>` is what gets stored in `agents.kind`
  and what `cvg agent spawn --runner` accepts.

## What this crate is NOT

- Not a process supervisor — that is `convergio-lifecycle::Supervisor`.
- Not the routing decision (which task → which runner) — that is
  `convergio-executor`'s job.
- Not the heartbeat / evidence transport — that is the agent itself
  calling back via the daemon HTTP API (the prompt instructs it).

## Tests

Subprocess tests are forbidden in this crate's unit tests — they
would tie the test runner to the operator's vendor logins and
charge real money. The unit tests assert on:

- `RunnerKind` parse + display round-trips
- `prompt::build` content (task id, evidence, graph context shape)
- `ClaudeRunner` / `CopilotRunner` argv shape (model flag,
  `--allow-all-tools`, etc.)
- `assert_cli_on_path` failure mode (hermetic — does not mutate
  the global `PATH`)

End-to-end tests against real `claude` / `copilot` belong in a
manual smoke script, not in CI.
