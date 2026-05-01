#!/usr/bin/env bash
# demo-two-sessions.sh — reproduce the PRD-001 KR1 dogfood live.
#
# Spins up two ephemeral session registrations (no actual Claude Code
# binary needed; the skill's HTTP calls are simulated) and prints
# `cvg status --agents` so the operator sees both sessions side by
# side. The intent is the README-promised live demo.
#
# Usage: bash examples/skills/cvg-attach/demo-two-sessions.sh
#
# Requires: cvg in PATH, the daemon running on 127.0.0.1:8420.
set -uo pipefail

DAEMON_URL="${CONVERGIO_API_URL:-http://127.0.0.1:8420}"
SUFFIX="$$-$(date +%s)"
ALPHA_ID="claude-code-demo-alpha-${SUFFIX}"
BETA_ID="claude-code-demo-beta-${SUFFIX}"

if ! curl -sf --max-time 2 "${DAEMON_URL}/v1/health" >/dev/null 2>&1; then
    echo "error: daemon not reachable at ${DAEMON_URL}" >&2
    echo "       start it with: cargo run -p convergio-server -- start" >&2
    exit 1
fi

now() { date -u +%Y-%m-%dT%H:%M:%SZ; }

register() {
    local id="$1"
    local host="$2"
    curl -sf -X POST -H 'content-type: application/json' \
        -d "{
            \"id\": \"${id}\",
            \"kind\": \"claude-code\",
            \"name\": \"session-${id}\",
            \"host\": \"${host}\",
            \"capabilities\": [\"edit\", \"read\", \"shell\", \"evidence-attach\"],
            \"metadata\": {
                \"tty\": \"demo\",
                \"pid\": ${RANDOM},
                \"cwd\": \"${PWD}\",
                \"session_started_at\": \"$(now)\"
            }
        }" \
        "${DAEMON_URL}/v1/agent-registry/agents" >/dev/null
}

announce() {
    local id="$1"
    curl -sf -X POST -H 'content-type: application/json' \
        -d "{
            \"topic\": \"system.session-events\",
            \"sender\": \"${id}\",
            \"payload\": {\"agent_id\": \"${id}\", \"kind\": \"agent.attached\"}
        }" \
        "${DAEMON_URL}/v1/system-messages" >/dev/null
}

echo "→ registering ${ALPHA_ID}..."
register ${ALPHA_ID} "host-demo-1"
announce ${ALPHA_ID}

echo "→ registering ${BETA_ID}..."
register ${BETA_ID} "host-demo-2"
announce ${BETA_ID}

echo
echo "→ cvg status --agents"
echo
if cvg status --help 2>&1 | grep -q -- '--agents'; then
    cvg status --agents --completed-limit 0
else
    echo "  (the cvg binary in PATH does not yet have --agents;"
    echo "   re-install it from this branch with:"
    echo "   cargo install --path crates/convergio-cli --force)"
    curl -sf "${DAEMON_URL}/v1/agent-registry/agents" \
        | python3 -m json.tool 2>/dev/null \
        | head -40 || true
fi

echo
echo "Both sessions are visible to the daemon."
echo "Their agent.attached presence messages are on the bus:"
echo
curl -sf "${DAEMON_URL}/v1/system-messages?topic=system.session-events&limit=10" \
    | python3 -m json.tool 2>/dev/null \
    | head -30 || true

echo
echo "Cleanup: retiring demo agents..."
curl -sf -X POST "${DAEMON_URL}/v1/agent-registry/agents/${ALPHA_ID}/retire" >/dev/null
curl -sf -X POST "${DAEMON_URL}/v1/agent-registry/agents/${BETA_ID}/retire" >/dev/null
echo "  ✓ done"
