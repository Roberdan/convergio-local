#!/usr/bin/env sh
set -eu

repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_dir"

cargo install --force --path crates/convergio-server
cargo install --force --path crates/convergio-cli

sync_shadowed_binary() {
  name="$1"
  cargo_bin="$HOME/.cargo/bin/$name"
  first_bin=$(command -v "$name" 2>/dev/null || true)
  if [ -n "$first_bin" ] && [ "$first_bin" != "$cargo_bin" ] && [ -w "$(dirname "$first_bin")" ]; then
    cp "$first_bin" "$first_bin.bak"
    cp "$cargo_bin" "$first_bin"
  fi
}

sync_shadowed_binary convergio
sync_shadowed_binary cvg

cat <<'MSG'

Installed:
  convergio  local daemon
  cvg        local CLI

Start:
  cvg setup
  convergio start

In another terminal:
  cvg doctor
  cvg health
  cvg demo
MSG
