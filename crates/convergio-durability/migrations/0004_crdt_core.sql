-- CRDT foundation for local multi-agent state.
--
-- v0.1 does not sync over the network, but every mergeable field needs a
-- durable actor/op model before public release.

CREATE TABLE IF NOT EXISTS crdt_actors (
    actor_id       TEXT PRIMARY KEY,
    kind           TEXT NOT NULL,            -- local|imported
    display_name   TEXT,
    is_local       INTEGER NOT NULL DEFAULT 0,
    created_at     TEXT NOT NULL,
    last_seen_at   TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_crdt_actors_single_local
    ON crdt_actors (is_local)
    WHERE is_local = 1;

CREATE TABLE IF NOT EXISTS crdt_ops (
    actor_id     TEXT NOT NULL REFERENCES crdt_actors(actor_id),
    counter      INTEGER NOT NULL,
    entity_type  TEXT NOT NULL,
    entity_id    TEXT NOT NULL,
    field_name   TEXT NOT NULL,
    crdt_type    TEXT NOT NULL,
    op_kind      TEXT NOT NULL,
    value        TEXT NOT NULL,
    hlc          TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    PRIMARY KEY (actor_id, counter)
);

CREATE INDEX IF NOT EXISTS idx_crdt_ops_entity
    ON crdt_ops (entity_type, entity_id, field_name);

CREATE TABLE IF NOT EXISTS crdt_cells (
    entity_type  TEXT NOT NULL,
    entity_id    TEXT NOT NULL,
    field_name   TEXT NOT NULL,
    crdt_type    TEXT NOT NULL,
    value        TEXT NOT NULL,
    clock        TEXT NOT NULL,
    conflict     TEXT,
    updated_at   TEXT NOT NULL,
    PRIMARY KEY (entity_type, entity_id, field_name)
);

CREATE TABLE IF NOT EXISTS crdt_row_clocks (
    entity_type  TEXT NOT NULL,
    entity_id    TEXT NOT NULL,
    clock        TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    PRIMARY KEY (entity_type, entity_id)
);
