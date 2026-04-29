# Agent instruction guidelines

Status: active.

Purpose: define how Convergio writes Markdown instructions, prompts,
plans, and folder context for AI coding agents.

## External basis

These rules are based on current vendor guidance and our own repo
experience:

- Claude Code loads `CLAUDE.md` files as persistent context; more specific
  files override broader ones, concise files are followed more reliably,
  and files over roughly 200 lines cost context.
- GitHub Copilot supports repository-wide instructions,
  path-specific instructions, and nearest `AGENTS.md` files for agents.
- Cursor-style rules favor scoped, explicit, path-aware context over one
  giant global prompt.

## Format rules

Agent docs must be written for execution, not persuasion.

Required properties:

| Property | Rule |
|----------|------|
| Scope | one repo, folder, crate, protocol, capability, or plan |
| Length | short enough to fit in working context; split by path when long |
| Headings | stable, predictable, grep-friendly |
| Language | imperative, concrete, testable |
| Paths | explicit relative paths |
| Commands | exact commands, preconditions, postconditions |
| Status | active/proposed/deprecated when decisions can change |
| Conflicts | say which file/rule wins |

Avoid:

- long motivational prose;
- duplicated root context in subfolders;
- vague rules such as "write good code";
- hidden assumptions;
- instructions that conflict across `AGENTS.md`, `CLAUDE.md`, Copilot, or
  Cursor rules;
- prompt-only behavior that should be enforced by daemon APIs or tests.

## AGENTS.md schema

Each local `AGENTS.md` should use this shape:

```markdown
# AGENTS.md — <scope>

For repo-wide rules see <relative-root-link>.

<one sentence responsibility>

## Invariants

- ...

## Do not

- ...

## Tests

- ...
```

Only add sections that change agent behavior. Do not fill templates for
their own sake.

## CLAUDE.md rule

When a folder needs Claude-specific discovery, `CLAUDE.md` should be a
symlink or short pointer to `AGENTS.md`.

Do not maintain separate Claude instructions unless an ADR explains why
the behavior must diverge.

## Plan file schema

Repo plans under `docs/plans/` should include:

```yaml
---
type: Plan
status: Active
owner: Convergio
updated: YYYY-MM-DD
source_of_truth: repo
---
```

Body:

1. `# <plan title>`
2. `## Objective`
3. `## Current state`
4. `## Invariants`
5. `## Phase order`
6. `## Task graph`
7. `## Acceptance criteria`
8. `## Validation`
9. `## Links`

Task IDs in plan files should match SQL todo IDs when they exist.

## Prompt/context packet rule

Context packets for workers should include only:

- plan summary;
- task objective;
- constraints;
- allowed resources;
- relevant prior evidence/messages;
- required output/evidence;
- nearest folder instructions.

Do not paste full conversation transcripts into worker prompts.
