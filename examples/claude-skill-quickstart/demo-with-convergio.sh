#!/usr/bin/env bash
# Demo: the same loop, every claim goes through Convergio.
#
# This script tells two stories side by side:
#
#   Plan A: agent submits work that contains a TODO marker.
#           Convergio refuses with HTTP 409, audit row written.
#
#   Plan B: agent submits clean work with all required evidence.
#           Submit accepted, Thor validates, task flips to done,
#           audit chain verifies.
#
# Two plans (not two tasks on the same plan) because Convergio's
# evidence store is append-only: once dirty evidence is attached, the
# task carries it forever. The audit chain integrity is the reason —
# we cannot rewrite history just because we want a clean retry.
#
# Read this script top to bottom. It is the documentation.

set -euo pipefail

URL="${CONVERGIO_URL:-http://127.0.0.1:8420}"
CVG="${CVG:-cvg}"

cyan="\033[36m"; red="\033[31m"; green="\033[32m"; dim="\033[2m"; reset="\033[0m"
say()    { printf "${cyan}%s${reset}\n" "$*"; }
ok()     { printf "${green}%s${reset}\n" "$*"; }
bad()    { printf "${red}%s${reset}\n" "$*"; }
faint()  { printf "${dim}%s${reset}\n" "$*"; }

if ! curl -fsS "$URL/v1/health" >/dev/null 2>&1; then
    bad "Convergio daemon is not reachable at $URL."
    bad "Start it with: cvg service start  (or: convergio start)"
    exit 1
fi

TEST_OK='{"warnings_count":0,"errors_count":0,"failures":[],"command":"cargo test"}'

#
# ─── Plan A: the dirty path ──────────────────────────────────────
#

say "[A1] cvg plan create  (Plan A — will refuse dirty work)"
PLAN_A=$("$CVG" plan create "claude-skill-quickstart-dirty" \
    --description "60-second demo, the dirty path" \
    --project "examples" \
    --output plain)
faint "      plan_id = $PLAN_A"

say "[A2] cvg task create  (requires code + test evidence)"
TASK_A=$("$CVG" task create "$PLAN_A" "implement handler" \
    --description "first attempt, will contain a TODO marker" \
    --evidence-required code,test \
    --output plain)
faint "      task_id = $TASK_A"
"$CVG" task transition "$TASK_A" in-progress \
    --agent-id demo-agent --output plain >/dev/null

say "[A3] agent attaches DIRTY evidence (TODO in the code diff)"
DIRTY_DIFF='{"diff":"// TODO: wire this later\nfn handler() {}"}'
faint "      code payload: $DIRTY_DIFF"
curl -fsS -X POST "$URL/v1/tasks/$TASK_A/evidence" \
    -H 'Content-Type: application/json' \
    -d "{\"kind\":\"code\",\"payload\":$DIRTY_DIFF,\"exit_code\":0}" \
    >/dev/null
curl -fsS -X POST "$URL/v1/tasks/$TASK_A/evidence" \
    -H 'Content-Type: application/json' \
    -d "{\"kind\":\"test\",\"payload\":$TEST_OK,\"exit_code\":0}" \
    >/dev/null

say "[A4] cvg task transition submitted  -> expect 409 gate_refused"
HTTP=$(curl -s -o /tmp/cvg-resp -w "%{http_code}" \
    -X POST "$URL/v1/tasks/$TASK_A/transition" \
    -H 'Content-Type: application/json' \
    -d "{\"target\":\"submitted\",\"agent_id\":\"demo-agent\"}")
if [ "$HTTP" = "409" ]; then
    ok    "      HTTP $HTTP — Convergio refused the dirty work"
    faint "      response: $(cat /tmp/cvg-resp)"
else
    bad "      HTTP $HTTP — expected 409, got something else"
    cat /tmp/cvg-resp; exit 1
fi
"$CVG" task transition "$TASK_A" failed \
    --agent-id demo-agent --output plain >/dev/null
faint "      retired the task to failed (audit row preserved)"

#
# ─── Plan B: the clean path ──────────────────────────────────────
#

echo
say "[B1] cvg plan create  (Plan B — clean attempt)"
PLAN_B=$("$CVG" plan create "claude-skill-quickstart-clean" \
    --description "60-second demo, the clean path" \
    --project "examples" \
    --output plain)
faint "      plan_id = $PLAN_B"

say "[B2] cvg task create  (requires code + test evidence)"
TASK_B=$("$CVG" task create "$PLAN_B" "implement handler" \
    --description "second attempt, no debt this time" \
    --evidence-required code,test \
    --output plain)
faint "      task_id = $TASK_B"
"$CVG" task transition "$TASK_B" in-progress \
    --agent-id demo-agent --output plain >/dev/null

say "[B3] agent attaches CLEAN evidence"
CLEAN_DIFF='{"diff":"fn handler() -> i32 { 42 }"}'
faint "      code payload: $CLEAN_DIFF"
curl -fsS -X POST "$URL/v1/tasks/$TASK_B/evidence" \
    -H 'Content-Type: application/json' \
    -d "{\"kind\":\"code\",\"payload\":$CLEAN_DIFF,\"exit_code\":0}" \
    >/dev/null
curl -fsS -X POST "$URL/v1/tasks/$TASK_B/evidence" \
    -H 'Content-Type: application/json' \
    -d "{\"kind\":\"test\",\"payload\":$TEST_OK,\"exit_code\":0}" \
    >/dev/null
"$CVG" task transition "$TASK_B" submitted \
    --agent-id demo-agent --output plain >/dev/null
ok    "      task is now submitted"

say "[B4] cvg validate <plan>  (Thor is the only path to done — ADR-0011)"
VERDICT=$("$CVG" validate "$PLAN_B" --output json 2>/dev/null | jq -r .verdict)
if [ "$VERDICT" = "pass" ]; then
    ok "      verdict: pass"
else
    bad "      verdict: $VERDICT (expected pass)"; exit 1
fi
STATUS=$("$CVG" task get "$TASK_B" --output json 2>/dev/null | jq -r .status)
ok        "      task status: $STATUS"

#
# ─── audit chain ─────────────────────────────────────────────────
#

echo
AUDIT=$(curl -fsS "$URL/v1/audit/verify")
ok "audit chain: $(echo "$AUDIT" | jq -r '"\(.checked) entries, ok=\(.ok)"')"
echo
ok "Outcome:"
ok "  Plan A — dirty work refused with audit row, task retired to failed."
ok "  Plan B — clean work submitted, Thor promoted submitted -> done."
ok "  The agent never set 'done' itself; only Thor can (ADR-0011)."
