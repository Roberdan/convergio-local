---
id: 0030
status: accepted
date: 2026-05-02
topics: [release, versioning, workspace, crates]
related_adrs: [0015, 0029]
touches_crates: []
last_validated: 2026-05-02
---

# 0030. Use one product version plus per-crate impact tracking

- Status: accepted
- Date: 2026-05-02
- Deciders: Roberdan
- Tags: release, versioning, workspace

## Context and Problem Statement

All Convergio crates currently inherit the workspace package version.
That makes releases simple, but it hides which crate actually changed in
a given release. The question: should every crate carry an independent
semantic version, or should Convergio keep one product version and track
crate-level impact elsewhere?

Convergio is currently shipped as one local daemon/CLI platform. Most
crates are internal city services: durability, bus, lifecycle, graph,
server, CLI, TUI, executor, planner and Thor are valuable because they
are wired together. Some crates (`convergio-api`, `convergio-mcp`, and
possibly `convergio-cli`) expose compatibility surfaces, but they are
still released from this repository as part of the same product.

## Decision Drivers

- **Operational clarity.** Releases should communicate both the product
  version and the crate that changed.
- **Low ceremony.** Independent semver for every internal crate adds
  bookkeeping without making the integrated daemon safer.
- **Compatibility honesty.** Agent-facing contracts need explicit
  compatibility notes even when the product version is shared.
- **ADR-0015.** Derived docs and indexes should describe state instead
  of relying on humans to keep scattered version notes aligned.

## Considered Options

1. **Independent semver for every crate.** Precise, but heavy. It makes
   internal crate changes look externally consumable and complicates
   release automation.
2. **One product version only.** Simple, but too coarse: a release can
   change only `convergio-tui` or `convergio-mcp` and the changelog does
   not make that obvious enough.
3. **Hybrid: one product version plus per-crate impact tracking
   (chosen).** Keep `[workspace.package].version` as the release
   version, and require PR/release notes to list changed crates,
   compatibility surfaces, migrations, docs and ADRs.

## Decision Outcome

Chosen option **(3): one product version plus per-crate impact
tracking**.

Convergio releases use one workspace product version. Internal crates do
not get independent version bumps just because they changed. Each PR and
release note should identify:

- changed crates;
- public API, CLI, MCP or HTTP contract impact;
- migrations or persistent data impact;
- related ADRs;
- validation profile.

Independent crate semver becomes appropriate only when a crate is
published or supported as a standalone external dependency. If that
happens, create a new ADR for that crate or crate family.

## Consequences

- **Positive.** The product remains easy to release and reason about as
  one local runtime.
- **Positive.** Reviewers and users still see crate-level impact in PRs,
  release notes and generated documentation.
- **Positive.** Compatibility-sensitive crates can be called out without
  forcing every internal crate into a fake independent release train.
- **Negative.** Consumers cannot infer crate churn from `Cargo.toml`
  versions alone. They must read release notes or generated impact
  tables.

## Standard process

For future crate changes:

1. Keep crate versions inherited from `[workspace.package].version`.
2. Add explicit PR/release notes for changed crates and compatibility
   surfaces.
3. Update root docs when product behavior changes.
4. Update crate-local docs when local responsibility or invariants
   change.
5. Regenerate derived documentation before pushing.

## Validation

- Root `Cargo.toml` uses one `[workspace.package].version`.
- `release-please-config.json` releases the repository as the
  `convergio` component.
- `README.md` documents crate-by-crate value separately from versioning.
