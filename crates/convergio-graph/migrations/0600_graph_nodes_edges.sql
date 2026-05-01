-- ADR-0014. Code-graph layer for Tier-3 context retrieval.
--
-- Migration range 600-699 reserved for convergio-graph (ADR-0003).
-- Schema is intentionally narrow: nodes + edges, both append-on-rebuild.
-- A "stale node" is detected via the source file mtime, not by complex
-- versioning. The parser is responsible for replacing the row when it
-- re-runs against a stale file.

CREATE TABLE IF NOT EXISTS graph_nodes (
    id           TEXT PRIMARY KEY,    -- stable hash of (kind, crate, path, name, optional span)
    kind         TEXT NOT NULL,       -- crate | module | item | adr | doc
    name         TEXT NOT NULL,
    file_path    TEXT,                -- NULL for adr/doc-only nodes
    crate_name   TEXT NOT NULL,       -- '__docs__' for non-code nodes
    item_kind    TEXT,                -- struct | enum | fn | trait | impl | const | type | macro (NULL when kind != 'item')
    span_start   INTEGER,             -- byte offset, NULL for non-code
    span_end     INTEGER,
    last_parsed  TEXT NOT NULL,       -- ISO-8601 UTC timestamp of last parse
    source_mtime TEXT NOT NULL        -- file mtime at parse time (for staleness check)
);

CREATE INDEX IF NOT EXISTS idx_graph_nodes_file
    ON graph_nodes(file_path);

CREATE INDEX IF NOT EXISTS idx_graph_nodes_crate
    ON graph_nodes(crate_name, kind);

CREATE TABLE IF NOT EXISTS graph_edges (
    src      TEXT NOT NULL REFERENCES graph_nodes(id) ON DELETE CASCADE,
    dst      TEXT NOT NULL REFERENCES graph_nodes(id) ON DELETE CASCADE,
    kind     TEXT NOT NULL,           -- uses | declares | re_exports | claims | mentions | depends_on
    weight   INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (src, dst, kind)
);

CREATE INDEX IF NOT EXISTS idx_graph_edges_dst
    ON graph_edges(dst, kind);

CREATE INDEX IF NOT EXISTS idx_graph_edges_kind
    ON graph_edges(kind);
