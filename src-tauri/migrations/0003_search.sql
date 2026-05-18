CREATE VIRTUAL TABLE search_index USING fts5(
    source UNINDEXED,
    source_id UNINDEXED,
    title UNINDEXED,
    content,
    created_at UNINDEXED,
    tokenize = 'unicode61 remove_diacritics 2'
);

CREATE TRIGGER messages_after_insert AFTER INSERT ON messages BEGIN
    INSERT INTO search_index(source, source_id, title, content, created_at)
    VALUES (
        'message',
        NEW.id,
        '',
        NEW.content,
        NEW.created_at
    );
END;

CREATE TRIGGER messages_after_delete AFTER DELETE ON messages BEGIN
    DELETE FROM search_index WHERE source = 'message' AND source_id = OLD.id;
END;

CREATE TRIGGER tasks_after_insert AFTER INSERT ON tasks BEGIN
    INSERT INTO search_index(source, source_id, title, content, created_at)
    VALUES (
        'task',
        NEW.id,
        NEW.title,
        NEW.title || ' ' || COALESCE(NEW.description, ''),
        NEW.created_at
    );
END;

CREATE TRIGGER tasks_after_delete AFTER DELETE ON tasks BEGIN
    DELETE FROM search_index WHERE source = 'task' AND source_id = OLD.id;
END;

CREATE TRIGGER sessions_after_insert AFTER INSERT ON sessions BEGIN
    INSERT INTO search_index(source, source_id, title, content, created_at)
    VALUES (
        'session',
        NEW.id,
        '',
        NEW.raw_input,
        NEW.created_at
    );
END;

CREATE TRIGGER sessions_after_update AFTER UPDATE OF ai_summary ON sessions
WHEN NEW.ai_summary IS NOT NULL
BEGIN
    UPDATE search_index
    SET content = NEW.raw_input || ' ' || NEW.ai_summary
    WHERE source = 'session' AND source_id = NEW.id;
END;

CREATE TRIGGER sessions_after_delete AFTER DELETE ON sessions BEGIN
    DELETE FROM search_index WHERE source = 'session' AND source_id = OLD.id;
END;

INSERT INTO search_index(source, source_id, title, content, created_at)
SELECT 'message', id, '', content, created_at FROM messages;

INSERT INTO search_index(source, source_id, title, content, created_at)
SELECT 'task', id, title, title || ' ' || COALESCE(description, ''), created_at FROM tasks;

INSERT INTO search_index(source, source_id, title, content, created_at)
SELECT
    'session',
    id,
    '',
    raw_input || ' ' || COALESCE(ai_summary, ''),
    created_at
FROM sessions;
