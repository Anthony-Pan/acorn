CREATE TABLE pins (
    id TEXT PRIMARY KEY,
    conversation_id TEXT,
    message_id TEXT,
    label TEXT NOT NULL DEFAULT 'Pinned',
    content TEXT NOT NULL,
    x REAL,
    y REAL,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_pins_created_at ON pins(created_at DESC);
