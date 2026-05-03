-- 0009_task_timing.sql
--
-- ADR-0031: materialised cache columns for task timing.
--
-- The audit log already records every transition, so duration is
-- always reconstructable. The cache columns just save the dashboard
-- and `cvg session resume` from joining audit_log on every render.
-- They are written in the same transaction as the transition (see
-- `Durability::transition_task` in facade_transitions.rs) so they
-- can never disagree with the audit row that documents the change.

ALTER TABLE tasks ADD COLUMN started_at TEXT;       -- RFC3339 first time the task entered in_progress
ALTER TABLE tasks ADD COLUMN ended_at   TEXT;       -- RFC3339 most recent transition into a terminal state (done, failed, cancelled)
ALTER TABLE tasks ADD COLUMN duration_ms INTEGER;   -- ended_at - started_at in milliseconds, NULL until ended

ALTER TABLE plans ADD COLUMN started_at TEXT;
ALTER TABLE plans ADD COLUMN ended_at   TEXT;
ALTER TABLE plans ADD COLUMN duration_ms INTEGER;

CREATE INDEX IF NOT EXISTS idx_tasks_timing ON tasks(started_at, ended_at);
CREATE INDEX IF NOT EXISTS idx_plans_timing ON plans(started_at, ended_at);
