# Contributing

Thanks for considering a contribution. Convergio is small and opinionated.
Read [AGENTS.md](./AGENTS.md) and [CONSTITUTION.md](./CONSTITUTION.md)
before writing code.

## Setup

```bash
git clone https://github.com/Roberdan/convergio
cd convergio
cargo build --workspace
cargo test --workspace
```

The toolchain is pinned in `rust-toolchain.toml`. Lefthook installs git
hooks (`fmt`, `clippy`, file-size guard, commitlint):

```bash
brew install lefthook && lefthook install   # macOS
# or: go install github.com/evilmartians/lefthook@latest
```

## Workflow

1. Branch off `main`. Use a worktree if you'll have parallel work:
   `git worktree add ../convergio-feature -b feat/short-name main`.
2. Implement + test.
3. Run the local CI bundle before pushing:
   ```bash
   cargo fmt --all -- --check
   RUSTFLAGS="-Dwarnings" cargo clippy --workspace --all-targets -- -D warnings
   RUSTFLAGS="-Dwarnings" cargo test --workspace
   ```
4. Commit using conventional commits with a crate scope
   (see `commitlint.config.js`):
   ```
   feat(durability): add audit hash chain
   fix(server): return 409 on gate refusal
   docs(repo): expand AGENTS.md request lifecycle section
   ```
5. Open a PR. Fill out all sections of the PR template — Problem,
   Why, What changed, Validation, Impact, **Files touched**. CI
   rejects PRs that don't.

## Merge order and the PR queue

When several PRs are open in parallel, merge in dependency order so
no PR force-pushes someone else's reviewable diff out from under
them. Suggested rules:

- A PR with **zero conflicts** in its `Files touched` manifest can
  merge any time once CI is green and the branch is up-to-date.
- A PR that overlaps another open PR's `Files touched` block waits
  for the other to merge, then **rebases on the new main** before
  its CI runs.
- The most invasive PR (the one rebased on top of every other) goes
  last. Mark it with `Depends on PR #N` lines in `Files touched`.

GitHub merge queue is the long-term automation for this rule —
enable it on `main` via the repository settings page (Settings →
Branches → Branch protection rule for `main` → Require merge queue).
The free public-repo plan supports it. Once enabled, the suggested
merge method stays `Merge commit` (CONSTITUTION § hard-rule:
`allow_squash_merge=false`, `allow_rebase_merge=false` — squash
loses parallel-agent history, rebase rewrites it).

Until merge queue is wired in, the local helper at
`scripts/cleanup-local-branches.sh` keeps the working tree tidy
after each `gh pr merge --delete-branch`.

## Code rules

| Rule | Reason |
|------|--------|
| Max **300 lines** per `*.rs` file | Forces module decomposition; CI-enforced |
| Every `pub` item has a `///` doc comment | Rustdoc + agent legibility |
| Every `lib.rs` opens with `//!` block | Crate purpose + entry-point map |
| No `unwrap()` / `panic!()` in production code | Use `Result` and `thiserror`/`anyhow` |
| Tests for every `pub fn` and every bug fix | Regression protection |
| New gate / new endpoint → ADR | `docs/adr/NNNN-short-title.md` |
| New DB field → migration in same PR | Reviewers reject schema PRs without one |

## Test conventions

- **Unit tests**: inline `#[cfg(test)] mod tests {}` in the same file.
- **Integration tests**: in `crates/<name>/tests/`, one feature per file
  (≤200 lines), shared helpers in `tests/common/mod.rs`.
- **E2E tests**: workspace-level `tests/` boot the server in-process.
- **Fixtures**: `crates/<name>/tests/fixtures/` (data) and
  `tests/snapshots/` (insta).
- **Never assert exact counts** for system-wide data — use `>=`. Exact
  counts only for test-scoped data.
- **Never hardcode the version** — use `env!("CARGO_PKG_VERSION")`.

## Documentation

- Rustdoc on every `pub` item. Treat `cargo doc --workspace --no-deps`
  as part of the build.
- Per-crate `README.md` synced from `lib.rs` `//!` via `cargo rdme` (run
  `cargo rdme --check` in CI).
- ADRs in `docs/adr/`. Use the MADR template at `docs/adr/0000-template.md`.

## Reporting issues

Use GitHub Issues with one of: `bug`, `feature`, `docs`, `question`.
Security issues go through `SECURITY.md` (private disclosure).

## License

By contributing you agree that your contributions are licensed under the
Convergio Community License v1.3, the same as the project. See
[LICENSE](./LICENSE) — in particular the "Contributions" section
(Contributor License Grant).
