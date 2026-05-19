-- Speech-to-text provider configuration, mirroring `provider_configs` for AI providers.
-- `selected_language` is a BCP-47 locale tag (e.g. "en-US", "zh-CN") that the
-- provider should prefer when transcribing; NULL means auto-detect.
CREATE TABLE speech_provider_configs (
    provider_id TEXT PRIMARY KEY,
    enabled INTEGER NOT NULL DEFAULT 1,
    selected_language TEXT,
    last_used_at TEXT
);
