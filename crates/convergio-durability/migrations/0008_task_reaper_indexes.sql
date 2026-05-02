-- Speed up stale in-progress task scans used by the Layer 1 reaper.
--
-- The reaper has two mutually exclusive stale cases:
--   1. a recorded heartbeat older than the cutoff;
--   2. no heartbeat yet, but the task claim itself is older than the cutoff.
--
-- Keep both indexes partial so non-running tasks do not pay the write cost.
CREATE INDEX IF NOT EXISTS idx_tasks_reaper_heartbeat
ON tasks (status, last_heartbeat_at, id, agent_id)
WHERE status = 'in_progress' AND last_heartbeat_at IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_tasks_reaper_no_heartbeat
ON tasks (status, last_heartbeat_at, updated_at, id, agent_id)
WHERE status = 'in_progress' AND last_heartbeat_at IS NULL;
