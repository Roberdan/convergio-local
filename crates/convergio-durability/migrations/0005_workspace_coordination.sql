-- Workspace coordination foundation.
--
-- Worktrees remain execution sandboxes. These tables model canonical
-- resources, leases, sessions, patch proposals, merge queue items, and
-- workspace conflicts so agents can coordinate before touching files/Git.

CREATE TABLE IF NOT EXISTS workspace_resources (
    id          TEXT PRIMARY KEY,
    kind        TEXT NOT NULL, -- repo|file|directory|symbol|artifact|ci_lane
    project     TEXT,
    path        TEXT NOT NULL,
    symbol      TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_workspace_resources_identity
    ON workspace_resources (kind, IFNULL(project, ''), path, IFNULL(symbol, ''));

CREATE TABLE IF NOT EXISTS workspace_leases (
    id           TEXT PRIMARY KEY,
    resource_id  TEXT NOT NULL REFERENCES workspace_resources(id),
    task_id      TEXT,
    agent_id     TEXT NOT NULL,
    purpose      TEXT,
    status       TEXT NOT NULL, -- active|released|expired
    expires_at   TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    released_at  TEXT
);

CREATE INDEX IF NOT EXISTS idx_workspace_leases_active
    ON workspace_leases (resource_id, status, expires_at);

CREATE TABLE IF NOT EXISTS agent_sessions (
    id             TEXT PRIMARY KEY,
    agent_id       TEXT NOT NULL,
    task_id        TEXT,
    base_revision  TEXT,
    sandbox_path   TEXT,
    status         TEXT NOT NULL,
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS patch_proposals (
    id                 TEXT PRIMARY KEY,
    task_id            TEXT NOT NULL,
    agent_id           TEXT NOT NULL,
    base_revision      TEXT NOT NULL,
    patch              TEXT NOT NULL,
    file_hashes        TEXT NOT NULL,
    status             TEXT NOT NULL,
    created_at         TEXT NOT NULL,
    updated_at         TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS merge_queue (
    id                 TEXT PRIMARY KEY,
    patch_proposal_id  TEXT NOT NULL REFERENCES patch_proposals(id),
    status             TEXT NOT NULL,
    sequence           INTEGER NOT NULL,
    created_at         TEXT NOT NULL,
    updated_at         TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS workspace_conflicts (
    id                 TEXT PRIMARY KEY,
    resource_id         TEXT REFERENCES workspace_resources(id),
    lease_id            TEXT REFERENCES workspace_leases(id),
    patch_proposal_id   TEXT REFERENCES patch_proposals(id),
    kind                TEXT NOT NULL,
    status              TEXT NOT NULL,
    details             TEXT NOT NULL,
    created_at          TEXT NOT NULL,
    resolved_at         TEXT
);
