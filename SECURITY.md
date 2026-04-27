# Security Policy

## Supported versions

Pre-1.0 — only the latest minor version is supported.
Once we hit 1.0 the policy becomes "latest two minor versions".

## Reporting a vulnerability

Please do **not** open public GitHub issues for security vulnerabilities.

Email **roberdan@fightthestroke.org** with:

- A clear description of the vulnerability
- Reproducer code or steps
- Affected version(s)
- Your proposed severity assessment

You will get an acknowledgement within 72 hours and a status update within
2 weeks. We coordinate disclosure once a fix is available.

## Threat model snapshot

Convergio stores plan/task/evidence/audit data on behalf of agents. The
durability properties we promise:

| Property | Mechanism |
|----------|-----------|
| Tamper-evidence on audit log | SHA-256 hash chain, externally verifiable via `GET /v1/audit/verify` |
| State survives crash | DB transactions, gates run server-side |
| No silent state changes | Every transition is audited |
| Personal mode isolation | localhost bind only, no auth required |
| Team mode isolation | HMAC signature on every request, multi-org via `org_id` |

Things explicitly **out of scope** for the MVP:

- RBAC / fine-grained permissions
- Encryption at rest (rely on filesystem / Postgres TDE)
- Network-level DoS protection (rely on reverse proxy)
- Sandboxing of spawned agents (Layer 3 spawns with the privileges of the daemon user)

## HMAC signature scheme (team mode)

Header: `X-Convergio-Signature: <hex(hmac_sha256(secret, body))>`.
Replay protection: `X-Convergio-Timestamp` (must be within ±5 minutes
of server time, included in the signed payload).
