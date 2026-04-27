-- Convergio Layer 3 — agent processes.
--
-- Tracks long-running agent processes spawned by the supervisor. The
-- daemon does NOT own the process lifecycle outside the row — if the
-- daemon dies the OS reaps the children, the rows become orphans, and
-- a future cleanup tick will mark them `terminated`.

CREATE TABLE IF NOT EXISTS agent_processes (
    id            TEXT PRIMARY KEY,
    kind          TEXT NOT NULL,            -- e.g. 'claude-code', 'shell', 'python'
    command       TEXT NOT NULL,            -- argv0
    plan_id       TEXT,
    task_id       TEXT,
    pid           INTEGER,                  -- OS pid; NULL until spawn returns
    status        TEXT NOT NULL,            -- 'starting' | 'running' | 'exited' | 'failed'
    exit_code     INTEGER,
    last_heartbeat_at TEXT,
    started_at    TEXT NOT NULL,
    ended_at      TEXT
);

CREATE INDEX IF NOT EXISTS idx_agent_processes_status ON agent_processes (status);
CREATE INDEX IF NOT EXISTS idx_agent_processes_task ON agent_processes (task_id);
