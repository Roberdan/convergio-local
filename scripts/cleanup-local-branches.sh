#!/usr/bin/env bash
# Prune local branches whose upstream is gone (post-merge cleanup).
# Run after any `gh pr merge --delete-branch` to keep `git branch`
# tidy. CONSTITUTION § "Repo hygiene" reflex.
set -euo pipefail

git fetch --prune origin >/dev/null 2>&1
gone=$(git branch -vv | awk '/: gone]/ {print $1}')
if [ -z "$gone" ]; then
    echo "no stale branches"
    exit 0
fi
echo "pruning local branches whose upstream is gone:"
echo "$gone" | while IFS= read -r b; do
    git branch -D "$b"
done
