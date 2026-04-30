## Problem

<!-- What is wrong / missing today? Link the issue or describe in 1-3 sentences. -->

## Why

<!-- Why does this matter? Who is the user feeling the pain? Reference roadmap, ADR, or constituent rule if relevant. -->

## What changed

<!-- The diff in plain English. Bullet list of crate-scoped changes. -->

## Validation

<!-- How was this verified? Paste real output. -->
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] Manual smoke (paste curl output if HTTP-touching)

## Impact

<!-- Breaking change? New migration? New env var? Deployment notes? -->

## Files touched

<!--
Machine-readable manifest used by `cvg pr stack` and other tools to
compute conflict matrices across open PRs without parsing each diff.
List crate-scoped paths only, one per line. Paths must match
`git diff --name-only main...HEAD`.

Optional `Depends on PR #N` line declares an explicit dependency
(must merge after #N).
-->

```
<crate-or-folder>/<file>
<crate-or-folder>/<file>
```

<!-- Depends on PR #N (uncomment if applicable) -->
