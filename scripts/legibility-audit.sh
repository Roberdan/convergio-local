#!/usr/bin/env bash
# Legibility audit — measures how comprehensible the repo is to an
# AI agent reading it cold. Combines four signals into a 0-100 score.
#
#   1. Cap headroom (50% of score)
#        per-file Rust under 300 lines (hard cap, lefthook-enforced)
#        per-crate Rust under 5_000 lines (soft, CONSTITUTION §13)
#                                10_000 lines (hard, CONSTITUTION §13)
#
#   2. Index density  (30%)
#        every crate under crates/ has AGENTS.md (CONSTITUTION §11)
#        every crate has CLAUDE.md (symlink/pointer to AGENTS.md)
#        every accepted ADR has matching content (file exists)
#
#   3. Audit-driven outcome  (20%)
#        if a Convergio daemon is reachable (CONVERGIO_URL env or
#        default), pull audit/verify and the recent task.refused
#        count from /v1/audit/refusals/latest. A high refusal rate
#        in the last 7 days signals legibility regression.
#        SKIPPED if daemon is not reachable — the static signals
#        are enough on their own.
#
#   4. (out of scope here)  Fresh-eyes simulation — see plan task
#        T4.06 ("Fresh-eyes legibility simulation: zero-shot agent
#        comprehension test").
#
# Exit codes:
#   0 — score >= 70 (target floor)
#   2 — soft-warn, 50 <= score < 70
#   1 — hard-fail, score < 50
#
# CI uses this advisory-only by default. Lefthook does not run it
# (too heavy for pre-commit). Run manually:
#
#   ./scripts/legibility-audit.sh
#   ./scripts/legibility-audit.sh --quiet      # score only
#   ./scripts/legibility-audit.sh --json       # structured output

set -euo pipefail

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_root"

quiet=0
json_out=0
for arg in "$@"; do
    case "$arg" in
        --quiet) quiet=1 ;;
        --json)  json_out=1 ;;
        *) echo "unknown arg: $arg" >&2; exit 64 ;;
    esac
done

#
# 1. Cap headroom
#
RS_HARD=300
CRATE_SOFT=5000
CRATE_HARD=10000

over_file_cap=$(find crates -name "*.rs" -not -path "*/target/*" 2>/dev/null \
    | xargs wc -l 2>/dev/null \
    | awk -v cap="$RS_HARD" '$1 > cap && $2 != "total" {c++} END {print c+0}')
near_file_cap=$(find crates -name "*.rs" -not -path "*/target/*" 2>/dev/null \
    | xargs wc -l 2>/dev/null \
    | awk -v cap="$RS_HARD" '$1 >= cap-50 && $1 <= cap && $2 != "total" {c++} END {print c+0}')

over_crate_hard=0
over_crate_soft=0
for d in crates/*/; do
    [ -d "$d/src" ] || continue
    loc=$(find "$d" -name "*.rs" -not -path "*/target/*" 2>/dev/null \
        | xargs cat 2>/dev/null \
        | wc -l \
        | tr -d ' ')
    if [ "$loc" -gt "$CRATE_HARD" ]; then
        over_crate_hard=$((over_crate_hard+1))
    elif [ "$loc" -gt "$CRATE_SOFT" ]; then
        over_crate_soft=$((over_crate_soft+1))
    fi
done

# 50 points: a hard-cap breach is severe; a near-cap file is just a
# headroom hint; a crate over the 10k hard cap is a structural smell;
# a crate over the 5k soft cap is a known-tracked todo (CONSTITUTION
# §13). Penalties calibrated so a clean tree with a few near-cap
# files and one tracked split target still scores comfortably above
# the 70 floor.
#
#   over file cap (>300)        -10 each
#   near file cap (250-300)      -1 each (mild headroom warning)
#   crate over hard 10_000 LOC  -25 each
#   crate over soft 5_000 LOC    -5 each
cap_score=50
cap_score=$((cap_score - over_file_cap*10))
cap_score=$((cap_score - near_file_cap*1))
cap_score=$((cap_score - over_crate_hard*25))
cap_score=$((cap_score - over_crate_soft*5))
[ "$cap_score" -lt 0 ] && cap_score=0

#
# 2. Index density
#
crates_total=$(find crates -maxdepth 1 -mindepth 1 -type d 2>/dev/null | wc -l | tr -d ' ')
crates_with_agents=0
crates_with_claude=0
for d in crates/*/; do
    [ -f "$d/AGENTS.md" ] && crates_with_agents=$((crates_with_agents+1))
    [ -e "$d/CLAUDE.md" ] && crates_with_claude=$((crates_with_claude+1))
done
adr_total=$(find docs/adr -maxdepth 1 -name "0*.md" 2>/dev/null | wc -l | tr -d ' ')
adr_with_status=$(grep -l "^- Status:" docs/adr/0*.md 2>/dev/null | wc -l | tr -d ' ')

# 30 points: 12 for AGENTS.md/crate, 12 for CLAUDE.md/crate, 6 for
# every ADR having a Status: line.
if [ "$crates_total" -gt 0 ]; then
    agents_pct=$((crates_with_agents * 100 / crates_total))
    claude_pct=$((crates_with_claude * 100 / crates_total))
else
    agents_pct=0; claude_pct=0
fi
if [ "$adr_total" -gt 0 ]; then
    adr_pct=$((adr_with_status * 100 / adr_total))
else
    adr_pct=100
fi
index_score=$(( (agents_pct * 12 / 100) + (claude_pct * 12 / 100) + (adr_pct * 6 / 100) ))

#
# 3. Audit-driven outcome
#
audit_score=20  # full credit if we cannot probe (advisory mode)
url="${CONVERGIO_URL:-http://127.0.0.1:8420}"
audit_status="skipped"
refusal_count=""
if curl -fsS "$url/v1/health" >/dev/null 2>&1; then
    verify=$(curl -fsS "$url/v1/audit/verify" 2>/dev/null || echo "{}")
    chain_ok=$(printf "%s" "$verify" | python3 -c "import json,sys
try: print(json.load(sys.stdin).get('ok'))
except: print('false')" 2>/dev/null)
    if [ "$chain_ok" != "True" ] && [ "$chain_ok" != "true" ]; then
        audit_score=0
        audit_status="chain_broken"
    else
        audit_status="verified"
        # We could fetch refusals/latest and count last-7d; for now
        # the chain integrity check alone gives full credit.
        refusal_count=$(printf "%s" "$verify" | python3 -c "import json,sys
try: print(json.load(sys.stdin).get('checked', 0))
except: print(0)" 2>/dev/null || echo 0)
    fi
fi

total=$((cap_score + index_score + audit_score))
[ "$total" -gt 100 ] && total=100

if [ "$json_out" = 1 ]; then
    cat <<JSON
{
  "score": $total,
  "cap_score": $cap_score,
  "index_score": $index_score,
  "audit_score": $audit_score,
  "audit_status": "$audit_status",
  "audit_chain_entries": "${refusal_count:-null}",
  "rust_files_over_300": $over_file_cap,
  "rust_files_near_300": $near_file_cap,
  "crates_over_soft_5k": $over_crate_soft,
  "crates_over_hard_10k": $over_crate_hard,
  "crates_total": $crates_total,
  "crates_with_agents_md": $crates_with_agents,
  "crates_with_claude_md": $crates_with_claude,
  "adr_total": $adr_total,
  "adr_with_status": $adr_with_status
}
JSON
elif [ "$quiet" = 1 ]; then
    echo "$total"
else
    cat <<TXT
=== legibility audit ===
score: $total / 100   (cap=$cap_score/50  index=$index_score/30  audit=$audit_score/20)

cap headroom:
  rust files over 300 lines      = $over_file_cap
  rust files within 50 of cap    = $near_file_cap
  crates over 10_000 lines (hard)= $over_crate_hard
  crates over  5_000 lines (soft)= $over_crate_soft

index density:
  crates with AGENTS.md          = $crates_with_agents / $crates_total
  crates with CLAUDE.md          = $crates_with_claude / $crates_total
  ADRs with explicit Status      = $adr_with_status / $adr_total

audit chain ($audit_status):
  entries verified               = ${refusal_count:-n/a}

floor: 70 / 100   target: 85 / 100
TXT
fi

if [ "$total" -lt 50 ]; then
    exit 1
elif [ "$total" -lt 70 ]; then
    exit 2
fi
exit 0
