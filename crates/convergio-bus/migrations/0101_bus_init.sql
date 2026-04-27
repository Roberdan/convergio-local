-- Convergio Layer 2 — agent message bus.
--
-- Persistent topic + direct messaging scoped per plan. See
-- crates/convergio-bus/src/lib.rs for the model.

CREATE TABLE IF NOT EXISTS agent_messages (
    id            TEXT PRIMARY KEY,
    seq           INTEGER NOT NULL,            -- monotonic per database
    plan_id       TEXT NOT NULL,
    topic         TEXT NOT NULL,               -- e.g. 'task.done', or 'agent:agent-1' for direct
    sender        TEXT,                        -- agent id, or NULL for system
    payload       TEXT NOT NULL,               -- canonical JSON
    consumed_at   TEXT,                        -- RFC3339, NULL = unconsumed
    consumed_by   TEXT,
    created_at    TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_agent_messages_seq ON agent_messages (seq);
CREATE INDEX IF NOT EXISTS idx_agent_messages_plan_topic
    ON agent_messages (plan_id, topic, consumed_at);
