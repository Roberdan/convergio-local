-- Bus migration 0103: relax plan_id NOT NULL for `system.*` topic family.
--
-- Implements ADR-0024. Until v0.2.0, every bus message was scoped to a
-- plan_id. PRD-001 (Claude Code adapter) needs presence broadcasts
-- (agent.attached, agent.heartbeat, agent.idle, agent.detached) and
-- cross-plan coordination messages (handshake, presence-announce) that
-- have no plan home.
--
-- This migration:
--   1. Drops the `plan_id NOT NULL` constraint by table-rebuild
--      (SQLite cannot ALTER COLUMN drop NOT NULL in place).
--   2. Adds a CHECK constraint enforcing that `plan_id IS NULL` is only
--      allowed when `topic` starts with `system.`. Plan-scoped traffic
--      keeps the same invariant it always had.
--   3. Adds a partial index on `(topic, created_at)` filtered to
--      `plan_id IS NULL` so system-topic polls do not scan the full
--      plan-scoped traffic.
--
-- The migration preserves every existing row, every index, and the
-- monotonic `seq` allocation — `agent_message_sequence` is untouched.

CREATE TABLE agent_messages_new (
    id            TEXT PRIMARY KEY,
    seq           INTEGER NOT NULL,
    plan_id       TEXT,                            -- relaxed; NULL allowed only for system.* topics
    topic         TEXT NOT NULL,
    sender        TEXT,
    payload       TEXT NOT NULL,
    consumed_at   TEXT,
    consumed_by   TEXT,
    created_at    TEXT NOT NULL,
    CHECK (
        plan_id IS NOT NULL
        OR topic LIKE 'system.%'
    )
);

INSERT INTO agent_messages_new (
    id, seq, plan_id, topic, sender, payload,
    consumed_at, consumed_by, created_at
)
SELECT id, seq, plan_id, topic, sender, payload,
       consumed_at, consumed_by, created_at
FROM agent_messages;

DROP TABLE agent_messages;
ALTER TABLE agent_messages_new RENAME TO agent_messages;

CREATE UNIQUE INDEX idx_agent_messages_seq
    ON agent_messages (seq);

CREATE INDEX idx_agent_messages_plan_topic
    ON agent_messages (plan_id, topic, consumed_at);

CREATE INDEX idx_agent_messages_system_topic
    ON agent_messages (topic, created_at)
    WHERE plan_id IS NULL;
