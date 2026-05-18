CREATE TABLE conversations (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL DEFAULT 'Untitled',
    provider_id TEXT,
    model TEXT,
    created_at TEXT NOT NULL,
    last_message_at TEXT NOT NULL
);

CREATE INDEX idx_conversations_last_message_at ON conversations(last_message_at DESC);

CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'tool', 'system')),
    content TEXT NOT NULL,
    tool_calls TEXT,
    tool_call_id TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_messages_conversation_id ON messages(conversation_id, created_at);
