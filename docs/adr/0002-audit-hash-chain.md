---
id: 0002
status: accepted
date: 2026-04-26
topics: [layer-1, security, compliance]
related_adrs: []
touches_crates: []
last_validated: 2026-04-30
---

# 0002. Hash-chain the audit log for tamper-evidence

- Status: accepted
- Date: 2026-04-26
- Deciders: Roberto
- Tags: layer-1, security, compliance

## Context and Problem Statement

Customers in regulated AI (healthcare, finance) need an audit trail that:

1. Records every state transition with WHO did WHAT and WHEN.
2. Is **tamper-evident**: a malicious or buggy operator that mutates a row
   should be detectable by an external auditor.
3. Is **cheap**: not a blockchain, not Merkle proofs, not multi-party signing.

We want this property in the OSS core so local runs are auditable
without external infrastructure.

## Decision Drivers

- Compliance posture (HIPAA, SOC 2, FDA 21 CFR Part 11) needs an audit
  log that cannot be silently modified.
- Verification must be a cron job, not a workflow engine.
- We don't have a central key infrastructure and don't want one in MVP.

## Considered Options

1. **Plain audit table** (no chain) — fine for "what happened" queries,
   useless against tampering.
2. **Per-row signature** — every row signed with a server key. Strong but
   requires key rotation, doesn't detect deletions in the middle.
3. **Hash-chained log** — each row's `hash = sha256(prev_hash || canonical_json(payload))`.
   Detects tampering and deletions. No keys.
4. **Merkle tree + checkpoint** — overkill for MVP, deferred.

## Decision Outcome

Chosen option: **3 — Hash-chained log**. Single column `hash` on every
`audit_log` row, chained from a fixed genesis (`0x00..0`).

### Verification protocol

```
GET /v1/audit/verify[?from=<id>&to=<id>]
```

Recomputes hashes server-side and returns `{ ok: bool, broken_at_id: ?id }`.
External cron calls this hourly and alarms on `ok == false`.

### Canonical JSON

To avoid false positives from formatting drift, the payload is canonicalized
before hashing: keys sorted lexicographically, no whitespace, numbers in
shortest form.

### Positive consequences

- Tamper-evidence with O(N) verification, no keys, no infrastructure.
- Works without external services.
- Easy to communicate ("hash chain like Git").

### Negative consequences

- Verification is O(N) — for very large audit logs we may need
  checkpointing. Tracked: future ADR.
- A row added between insert and chain-update is a bug surface.
  Mitigation: insert and chain-update happen in one DB transaction.

## Links

- Spec: [docs/spec/v3-durability-layer.md](../spec/v3-durability-layer.md) § "Layer 1 — Durability Core"
- Constitution: [CONSTITUTION.md](../../CONSTITUTION.md) § 7
