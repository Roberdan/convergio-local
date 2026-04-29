-- Local capability registry.
--
-- This records capabilities that have already been installed by a trusted
-- path. Download/install/package extraction are intentionally separate.

CREATE TABLE IF NOT EXISTS capabilities (
    name          TEXT PRIMARY KEY,
    version       TEXT NOT NULL,
    status        TEXT NOT NULL, -- installed|enabled|disabled|failed
    source        TEXT NOT NULL DEFAULT 'local',
    root_path     TEXT,
    manifest      TEXT NOT NULL,
    checksum      TEXT,
    signature     TEXT,
    installed_at  TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_capabilities_status ON capabilities (status);

CREATE TABLE IF NOT EXISTS capability_migrations (
    id               TEXT PRIMARY KEY,
    capability_name  TEXT NOT NULL REFERENCES capabilities(name) ON DELETE CASCADE,
    version          INTEGER NOT NULL,
    up_hash          TEXT NOT NULL,
    down_hash        TEXT,
    status           TEXT NOT NULL,
    applied_at       TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_capability_migrations_version
    ON capability_migrations (capability_name, version);
