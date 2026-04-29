# Security Policy

## Supported versions

Pre-1.0 — only the latest minor version is supported.

## Reporting a vulnerability

Please do **not** open public GitHub issues for security
vulnerabilities.

Email **roberdan@fightthestroke.org** with:

- a clear description of the vulnerability
- reproducer code or steps
- affected version(s)
- your proposed severity assessment

You will get an acknowledgement within 72 hours and a status update
within 2 weeks. We coordinate disclosure once a fix is available.

## Threat model snapshot

Convergio is a **single-user local daemon**. It stores plan, task,
evidence, message, process and audit data in a local SQLite database.

| Property | Mechanism |
|----------|-----------|
| Local-only default | daemon binds to `127.0.0.1:8420` |
| No external services | SQLite file at `~/.convergio/state.db` |
| Tamper-evident audit | SHA-256 hash chain, verifiable via `GET /v1/audit/verify` |
| No silent task completion | transitions run server-side gates before state changes |
| Crash-resistant state | persisted plans, tasks, evidence, messages and process rows |

## Important local safety notes

- Keep the daemon bound to localhost unless you fully understand the
  risk. The daemon refuses non-local bind addresses unless started with
  `--allow-non-local-bind`.
- `/v1/agents/spawn` starts processes with the daemon user's privileges.
  It is a local automation feature, not a sandbox.
- Evidence is untrusted input. Gates must inspect it defensively.
- Do not put secrets in evidence payloads, logs, task descriptions or
  command arguments.

## Out of scope for the local MVP

- network exposure hardening
- RBAC or accounts
- encryption beyond filesystem protections
- sandboxing of spawned agents
- denial-of-service protection beyond local process limits
