# Contributing

Thanks for considering a contribution. Convergio is small and opinionated.
Read [AGENTS.md](./AGENTS.md) and [CONSTITUTION.md](./CONSTITUTION.md)
before writing code.

## Setup

```bash
git clone https://github.com/Roberdan/convergioV3
cd convergioV3
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
5. Open a PR. Fill out all 5 sections of the PR template — CI rejects PRs
   that don't.

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
Apache 2.0 license, the same as the project.
