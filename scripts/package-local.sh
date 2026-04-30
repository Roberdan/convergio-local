#!/usr/bin/env sh
set -eu
export LC_ALL=C   # locale-stable sort/awk/grep across macOS / Linux CI (T1.19 / F27)

repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_dir"

cargo build --release --workspace

name="convergio-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m)"
rm -rf "dist/$name"
mkdir -p "dist/$name/bin"
cp target/release/convergio "dist/$name/bin/"
cp target/release/cvg "dist/$name/bin/"
cp target/release/convergio-mcp "dist/$name/bin/"
cp README.md LICENSE "dist/$name/"
tar -C dist -czf "dist/$name.tar.gz" "$name"
echo "dist/$name.tar.gz"
