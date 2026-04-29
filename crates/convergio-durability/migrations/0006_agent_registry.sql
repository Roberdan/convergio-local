-- Durable agent registry.
--
-- `agents` is the daemon-mediated identity registry for manual and
-- orchestrated workers. `agent_processes` remains Layer 3 OS supervision.

ALTER TABLE agents ADD COLUMN name TEXT;
ALTER TABLE agents ADD COLUMN host TEXT;
ALTER TABLE agents ADD COLUMN capabilities TEXT NOT NULL DEFAULT '[]';
ALTER TABLE agents ADD COLUMN current_task_id TEXT;
ALTER TABLE agents ADD COLUMN metadata TEXT NOT NULL DEFAULT '{}';

CREATE INDEX IF NOT EXISTS idx_agents_current_task ON agents (current_task_id);
CREATE INDEX IF NOT EXISTS idx_agents_kind ON agents (kind);
