# Plans

Durable project plans live here. They are committed history, not
session-only scratch notes.

## File types

| Type | Use |
|------|-----|
| `*.md` | agent-executable engineering plans and product roadmaps |
| `*.yaml` | daemon/planner input once a plan is meant to be consumed by code |

Inactive or superseded plans go in `docs/plans/archive/`.

## Required structure for Markdown plans

Use stable headings and compact tables so agents can execute the plan
without rereading the whole repo:

1. objective;
2. current state;
3. invariants;
4. phases;
5. task IDs;
6. dependencies;
7. acceptance criteria;
8. validation commands;
9. links to ADRs and implementation files.

Repo plans are the engineering source of truth for this codebase.
Obsidian notes may mirror or index them, but should link back here.
