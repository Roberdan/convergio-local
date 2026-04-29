# AGENTS.md — convergio-thor

For repo-wide rules see [../../AGENTS.md](../../AGENTS.md).

Thor is the reference validator over plans/tasks/evidence.

## Invariants

- Validation reports must be deterministic and explainable.
- Do not duplicate gate logic; call the owning layer or consume its
  results.
- A plan is not valid if tasks are completed without accepted evidence.
- Keep this crate provider-neutral.
