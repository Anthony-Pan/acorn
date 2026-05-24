CREATE TABLE canvases (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL DEFAULT 'Untitled canvas',
    content TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_canvases_updated_at ON canvases(updated_at DESC);
