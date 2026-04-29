-- Robust local message sequence allocation.

CREATE TABLE IF NOT EXISTS agent_message_sequence (
    id       INTEGER PRIMARY KEY CHECK (id = 1),
    next_seq INTEGER NOT NULL
);

INSERT INTO agent_message_sequence (id, next_seq)
SELECT 1, COALESCE(MAX(seq), 0) + 1
FROM agent_messages
WHERE NOT EXISTS (SELECT 1 FROM agent_message_sequence WHERE id = 1);
