# AGENTS.md — docs

For repo-wide rules see [../AGENTS.md](../AGENTS.md).

This folder is product memory. Documentation must not claim behavior that
is not implemented or explicitly marked as future work.

## Rules

- Keep vision, ADR, roadmap, and user docs consistent.
- Mark future behavior as future; do not phrase it as shipped.
- When adding an ADR, update `docs/adr/README.md`.
- Prefer one focused doc over scattering the same concept in many files.
- If an implementation changes the user workflow, update the relevant doc
  in the same PR.
- Follow `docs/agent-instruction-guidelines.md` for agent-optimized
  Markdown and prompt files.

## Current load-bearing docs

- `docs/vision.md` — product direction.
- `docs/multi-agent-operating-model.md` — how swarms use Convergio.
- `docs/agent-instruction-guidelines.md` — format rules for agent docs.
- `docs/agent-protocol.md` — MCP/tool loop for agents.
- `docs/adr/` — decisions that constrain implementation.
