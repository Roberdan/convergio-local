#!/usr/bin/env bash
# Reject PRs that add a new actionable F## row to the friction log
# without a matching entry in the "Daemon task mirror" section
# (closes F40 — keep one source of truth for outstanding work).
#
# Logic:
#   1. Find rows added vs `origin/main` to docs/plans/v0.2-friction-log.md
#      that match `^| F[0-9]+ |` and whose status cell is NOT
#      "accepted" — i.e. actionable rows.
#   2. For each such F##, require a row in the "Daemon task mirror"
#      table referencing both the F## label and a UUIDv4.
#   3. Exit 1 with a focused diagnostic when any are missing.
#
# Skip when the file itself is unchanged (most PRs).
#
# Exit codes:
#   0  clean (or N/A)
#   1  one or more new F## rows lack a daemon mirror row
#   2  malformed inputs (no friction log file present)

set -euo pipefail
export LC_ALL=C

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_root"

LOG_PATH="docs/plans/v0.2-friction-log.md"
BASE_REF="${BASE_REF:-origin/main}"

if [ ! -f "$LOG_PATH" ]; then
  echo "friction log not found at $LOG_PATH" >&2
  exit 2
fi

# Resolve base ref; if origin/main is unreachable (shallow clones in
# some CI configurations), fall back to HEAD~1.
if ! git rev-parse --verify "$BASE_REF" >/dev/null 2>&1; then
  BASE_REF=$(git rev-parse HEAD~1 2>/dev/null || echo "")
fi

if [ -z "$BASE_REF" ]; then
  echo "no base ref to diff against — skipping friction-log mirror check"
  exit 0
fi

# 1) Lines added to the friction log file in this branch.
added=$(git diff "$BASE_REF"...HEAD -- "$LOG_PATH" 2>/dev/null \
  | awk '/^\+\| F[0-9]+ \|/ {print substr($0,2)}' || true)

if [ -z "$added" ]; then
  echo "no new F## rows in $LOG_PATH — skipping mirror check"
  exit 0
fi

# 2) Build the set of F## labels that appear in the daemon mirror
#    table together with a UUID-shaped token. The mirror header is
#    "Daemon task mirror"; every row has the form
#    `| F## | <plan> | \`<uuid>\` | ...`.
mirror_labels=$(awk '
  /^## Daemon task mirror/ { in_mirror = 1; next }
  /^## / && in_mirror      { in_mirror = 0 }
  in_mirror && /^\| F[0-9A-Za-z-]+ \|/ {
    label = $2
    # accept UUID v4-shape token in row, e.g. ``<8>-<4>-<4>-<4>-<12>``
    if (match($0, /[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/)) {
      print label
    }
  }
' "$LOG_PATH" | sort -u)

# 3) For every newly added F##, require it to appear in the mirror set,
#    UNLESS the row's status column is exactly "accepted" (P3 by-design).
missing=""
while IFS= read -r row; do
  [ -z "$row" ] && continue
  label=$(echo "$row" | awk -F '|' '{gsub(/^ +| +$/,"",$2); print $2}')
  status=$(echo "$row" | awk -F '|' '{gsub(/^ +| +$/,"",$5); print $5}')
  case "$status" in
    accepted|"n/a (positive)") continue ;;
  esac
  if ! grep -qx "$label" <<< "$mirror_labels"; then
    missing="$missing $label"
  fi
done <<< "$added"

if [ -n "$missing" ]; then
  echo "FAIL: new actionable friction-log rows are missing a daemon mirror entry:" >&2
  for m in $missing; do echo "  - $m" >&2; done
  echo >&2
  echo "Fix: create a daemon task and add a row to the 'Daemon task mirror'" >&2
  echo "table in $LOG_PATH (see AGENTS.md § Friction log ↔ daemon mirror)." >&2
  exit 1
fi

echo "OK: every new F## row has a daemon mirror entry"
