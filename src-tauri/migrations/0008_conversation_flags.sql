ALTER TABLE conversations ADD COLUMN favorite INTEGER NOT NULL DEFAULT 0;
ALTER TABLE conversations ADD COLUMN archived_at TEXT;

CREATE INDEX idx_conversations_favorite ON conversations(favorite);
