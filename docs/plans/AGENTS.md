# AGENTS.md — docs/plans

For repo-wide rules see [../../AGENTS.md](../../AGENTS.md).

This folder stores durable project plans that agents can execute and
future maintainers can audit.

## Invariants

- Repo plans are committed history.
- Session plans are scratch unless copied here.
- Use kebab-case filenames.
- Markdown plans must include task IDs, dependencies, acceptance criteria,
  validation commands, and links to ADRs.
- If a plan is mirrored in Obsidian, the repo copy remains the
  engineering source of truth.

## Do not

- Do not store vague brainstorming here.
- Do not create a plan without status metadata.
- Do not let plan task IDs drift from SQL todo IDs once execution starts.
