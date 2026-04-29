#!/usr/bin/env sh
set -eu

repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_dir"

cargo install --path crates/convergio-server
cargo install --path crates/convergio-cli

cat <<'MSG'

Installed:
  convergio  local daemon
  cvg        local CLI

Start:
  convergio start

In another terminal:
  cvg health
  cvg demo
MSG
