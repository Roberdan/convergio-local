-- Convergio Layer 1 — initial schema.
--
-- SQLite schema. We use portable, simple types:
--   id        : TEXT (UUID v4 as string)
--   timestamps: TEXT (RFC3339 / ISO-8601, UTC) — sqlx::chrono handles both
--   payloads  : TEXT (canonical JSON string, written via serde_json)
--
CREATE TABLE IF NOT EXISTS plans (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    description TEXT,
    status      TEXT NOT NULL DEFAULT 'draft',  -- draft|active|completed|cancelled
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tasks (
    id                  TEXT PRIMARY KEY,
    plan_id             TEXT NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    wave                INTEGER NOT NULL DEFAULT 1,
    sequence            INTEGER NOT NULL DEFAULT 1,
    title               TEXT NOT NULL,
    description         TEXT,
    status              TEXT NOT NULL DEFAULT 'pending',  -- pending|in_progress|submitted|done|failed
    agent_id            TEXT,
    evidence_required   TEXT NOT NULL DEFAULT '[]',       -- JSON array of required evidence kinds
    last_heartbeat_at   TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_tasks_plan ON tasks (plan_id, wave, sequence);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks (status);

CREATE TABLE IF NOT EXISTS evidence (
    id           TEXT PRIMARY KEY,
    task_id      TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    kind         TEXT NOT NULL,           -- e.g. 'test_pass', 'pr_url', 'manual'
    payload      TEXT NOT NULL,           -- canonical JSON
    exit_code    INTEGER,
    created_at   TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_evidence_task ON evidence (task_id);

CREATE TABLE IF NOT EXISTS agents (
    id                 TEXT PRIMARY KEY,
    kind               TEXT NOT NULL,
    status             TEXT NOT NULL DEFAULT 'idle',     -- idle|working|unhealthy|terminated
    last_heartbeat_at  TEXT,
    created_at         TEXT NOT NULL,
    updated_at         TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_agents_status ON agents (status);

-- Append-only audit log. The hash chain is documented in ADR-0002.
CREATE TABLE IF NOT EXISTS audit_log (
    id            TEXT PRIMARY KEY,
    seq           INTEGER NOT NULL,        -- monotonic per database, used for chain order
    entity_type   TEXT NOT NULL,           -- 'plan' | 'task' | 'evidence' | 'agent'
    entity_id     TEXT NOT NULL,
    transition    TEXT NOT NULL,           -- e.g. 'plan.created', 'task.in_progress'
    payload       TEXT NOT NULL,           -- canonical JSON of the full event
    agent_id      TEXT,
    prev_hash     TEXT NOT NULL,
    hash          TEXT NOT NULL,
    created_at    TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_audit_seq ON audit_log (seq);
CREATE INDEX IF NOT EXISTS idx_audit_entity ON audit_log (entity_type, entity_id);
