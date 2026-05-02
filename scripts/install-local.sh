#!/usr/bin/env sh
set -eu
export LC_ALL=C   # locale-stable sort/awk/grep across macOS / Linux CI (T1.19 / F27)

repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_dir"

cargo install --force --path crates/convergio-server
cargo install --force --path crates/convergio-cli
cargo install --force --path crates/convergio-mcp

sync_shadowed_binary() {
  name="$1"
  cargo_bin="$HOME/.cargo/bin/$name"
  first_bin=$(command -v "$name" 2>/dev/null || true)
  if [ -n "$first_bin" ] && [ "$first_bin" != "$cargo_bin" ] && [ -w "$(dirname "$first_bin")" ]; then
    cp "$first_bin" "$first_bin.bak"
    tmp="$first_bin.tmp.$$"
    cp "$cargo_bin" "$tmp"
    mv "$tmp" "$first_bin"
  fi
}

sync_shadowed_binary convergio
sync_shadowed_binary cvg
sync_shadowed_binary convergio-mcp

# Install Git hooks so the file-size guard, fmt/clippy gates, and
# commitlint run on every commit. Without this every fresh clone
# silently bypasses CONSTITUTION § 13. Closes F31.
#
# F39: clean up any absolute core.hooksPath leftover from a previous
# install or from an ancestor folder rename — lefthook expects the
# default relative .git/hooks/ path, and an absolute override breaks
# the moment the repo is moved or renamed.
hooks_path=$(git config --get core.hooksPath 2>/dev/null || true)
case "$hooks_path" in
  /*)
    echo "info: clearing absolute core.hooksPath ($hooks_path)" >&2
    git config --unset core.hooksPath || true
    ;;
esac

if command -v lefthook >/dev/null 2>&1; then
  lefthook install
else
  cat <<'HINT' >&2

WARN: lefthook not on PATH — Git hooks NOT installed.
      Without them every commit skips fmt/clippy/file-size/commitlint
      gates locally (CI still catches them, but slow feedback).
      Install one of:
        brew install lefthook && lefthook install
        go install github.com/evilmartians/lefthook@latest && lefthook install
        npm install -g lefthook && lefthook install

HINT
fi

cat <<'MSG'

Installed:
  convergio  local daemon
  cvg        local CLI
  convergio-mcp  MCP bridge for agents

Start:
  cvg setup
  convergio start

In another terminal:
  cvg doctor
  cvg health
  cvg demo
MSG
