-- Robust local audit sequence allocation.

CREATE TABLE IF NOT EXISTS audit_sequence (
    id       INTEGER PRIMARY KEY CHECK (id = 1),
    next_seq INTEGER NOT NULL
);

INSERT INTO audit_sequence (id, next_seq)
SELECT 1, COALESCE(MAX(seq), 0) + 1
FROM audit_log
WHERE NOT EXISTS (SELECT 1 FROM audit_sequence WHERE id = 1);
