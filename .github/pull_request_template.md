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
