#!/usr/bin/env bash
# cvg-attach.sh — register the current session as a Convergio agent.
#
# Idempotent, non-blocking: warns once on stderr if the daemon is
# unreachable but always exits 0 so the SessionStart hook never
# blocks the user.
#
# Reads:
#   $CONVERGIO_API_URL  — defaults to http://127.0.0.1:8420
#   $USER, $HOSTNAME, $TTY, $PWD, $$ — session context
#
# Writes:
#   ~/.convergio/state/sessions/${PID}.agent — the new agent_id
#
# Emits:
#   POST /v1/agent-registry/agents — registration
#   POST /v1/plans/system/messages — agent.attached on
#                                    system.session-events
set -euo pipefail

DAEMON_URL="${CONVERGIO_API_URL:-http://127.0.0.1:8420}"
STATE_DIR="${HOME}/.convergio/state/sessions"
HOST_LABEL="${HOSTNAME:-$(hostname -s 2>/dev/null || echo unknown)}"
USER_LABEL="${USER:-unknown}"
PID="$$"

# 8 random hex chars; falls back to a timestamp if /dev/urandom is
# unavailable so tests in restricted environments still produce a
# deterministic-looking id.
random_hex() {
    if command -v openssl >/dev/null 2>&1; then
        openssl rand -hex 4
    elif [ -r /dev/urandom ]; then
        od -A n -t x1 -N 4 /dev/urandom | tr -d ' \n'
    else
        printf '%08x' "$(date +%s)"
    fi
}

AGENT_ID="claude-code-${USER_LABEL}-$(random_hex)"
SESSION_STARTED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# Build the NewAgent payload. `capabilities` matches the field name
# in `convergio_durability::store::agents::NewAgent`.
PAYLOAD=$(cat <<JSON
{
  "id": "${AGENT_ID}",
  "kind": "claude-code",
  "name": "claude-code-${USER_LABEL}-${PID}",
  "host": "${HOST_LABEL}",
  "capabilities": ["edit", "read", "shell", "evidence-attach"],
  "metadata": {
    "tty": "${TTY:-unknown}",
    "pid": ${PID},
    "cwd": "${PWD}",
    "session_started_at": "${SESSION_STARTED_AT}"
  }
}
JSON
)

# Pre-flight: if daemon is not reachable in 2s, surface a single
# warning and exit 0. The user's session continues without
# coordination — that is the documented degraded mode (PRD-001
# § Risks).
if ! curl -sf --max-time 2 "${DAEMON_URL}/v1/health" >/dev/null 2>&1; then
    echo "warning: Convergio daemon offline at ${DAEMON_URL}; coordination disabled" >&2
    exit 0
fi

# Register. 3 attempts with 500 ms backoff.
attempt=0
register_response=""
while [ "$attempt" -lt 3 ]; do
    if register_response=$(curl -sf --max-time 5 \
        -X POST \
        -H 'content-type: application/json' \
        -d "${PAYLOAD}" \
        "${DAEMON_URL}/v1/agent-registry/agents" 2>/dev/null); then
        break
    fi
    attempt=$((attempt + 1))
    sleep 0.5
done

if [ -z "${register_response}" ]; then
    echo "warning: Convergio agent registration failed after 3 attempts" >&2
    exit 0
fi

# Persist the agent id (idempotent: overwrite any stale row for
# this PID; the Stop hook reads this file to retire the right id).
mkdir -p "${STATE_DIR}"
printf '%s\n' "${AGENT_ID}" > "${STATE_DIR}/${PID}.agent"

# Publish presence on system.session-events (best-effort,
# silent on failure — a registered agent that didn't manage to
# publish presence is still better than no registration at all).
PRESENCE_PAYLOAD=$(cat <<JSON
{
  "topic": "system.session-events",
  "kind": "agent.attached",
  "payload": {
    "agent_id": "${AGENT_ID}",
    "kind": "claude-code",
    "host": "${HOST_LABEL}",
    "pid": ${PID},
    "cwd": "${PWD}",
    "started_at": "${SESSION_STARTED_AT}"
  }
}
JSON
)
curl -sf --max-time 2 \
    -X POST \
    -H 'content-type: application/json' \
    -d "${PRESENCE_PAYLOAD}" \
    "${DAEMON_URL}/v1/system-messages" >/dev/null 2>&1 || true

echo "Convergio agent registered: ${AGENT_ID}"
exit 0
