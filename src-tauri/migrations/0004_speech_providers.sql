-- Speech-to-text provider configuration, mirroring `provider_configs` for AI providers.
-- `selected_language` is a BCP-47 locale tag (e.g. "en-US", "zh-CN") that the
-- provider should prefer when transcribing; NULL means auto-detect.
-- IF NOT EXISTS so users who ran the earlier 0002_speech_providers.sql dev
-- build (since renamed to 0004) don't crash on next launch — the repair step
-- in db/mod.rs::repair_migration_history clears the stale 0002 row and lets
-- sqlx replay 0004 over the table that already exists.
CREATE TABLE IF NOT EXISTS speech_provider_configs (
    provider_id TEXT PRIMARY KEY,
    enabled INTEGER NOT NULL DEFAULT 1,
    selected_language TEXT,
    last_used_at TEXT
);
