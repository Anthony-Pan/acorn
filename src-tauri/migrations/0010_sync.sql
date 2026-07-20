-- Two-way calendar / reminders sync: accounts, per-task links, and cursors.
--
-- `acorn.db` stays the source of truth; providers (Apple Calendar, Apple
-- Reminders, Google Calendar, Google Tasks) are peers we reconcile against.

-- One row per connected account. `provider` is the system; `target_kind`
-- decides whether tasks sync as calendar events or as reminders/tasks.
CREATE TABLE sync_accounts (
    id              TEXT PRIMARY KEY,               -- uuid v4
    provider        TEXT NOT NULL,                  -- 'apple_calendar'|'apple_reminders'|'google_calendar'|'google_tasks'
    target_kind     TEXT NOT NULL,                  -- 'event' | 'reminder'
    account_label   TEXT NOT NULL,                  -- display name (Google email; Apple = 'Local')
    container_id    TEXT,                           -- selected calendarId / tasklist id / EKCalendar identifier
    container_name  TEXT,                           -- human label of the selected calendar/list
    enabled         INTEGER NOT NULL DEFAULT 1,
    has_token       INTEGER NOT NULL DEFAULT 0,     -- OAuth presence flag; token bytes live in the keychain. Apple = 0
    conflict_policy TEXT NOT NULL DEFAULT 'lww',    -- 'lww' | 'local_wins' | 'remote_wins'
    last_synced_at  TEXT,
    last_error      TEXT,
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX idx_sync_accounts_provider ON sync_accounts(provider, enabled);

-- One row per (local task <-> account) link. A task fans out to N accounts.
--
-- task_id is nullable with ON DELETE SET NULL (not CASCADE): deleting a task
-- leaves its already-tombstoned links intact so the engine can still push the
-- remote delete using remote_id after the domain row is gone.
CREATE TABLE sync_links (
    id                TEXT PRIMARY KEY,             -- uuid v4
    task_id           TEXT,
    account_id        TEXT NOT NULL,
    remote_id         TEXT,                         -- provider id; NULL until first push confirmed
    remote_etag       TEXT,                         -- Google etag; NULL for EventKit
    remote_updated    TEXT,                         -- RFC3339 from provider
    content_hash      TEXT,                         -- hash of the synced field set; suppresses echo loops
    dedupe_key        TEXT,                         -- client key recoverable on the remote (retry safety)
    local_rev         INTEGER NOT NULL DEFAULT 0,   -- bumped on every local edit (durable dirty flag)
    synced_local_rev  INTEGER NOT NULL DEFAULT 0,   -- local_rev at last successful push
    sync_state        TEXT NOT NULL DEFAULT 'pending_create',
        -- 'pending_create'|'pending_update'|'pending_delete'|'synced'|'conflict'|'error'
    last_synced_at    TEXT,
    last_error        TEXT,
    retry_count       INTEGER NOT NULL DEFAULT 0,
    deleted_at        TEXT,                         -- tombstone marker (NULL = live)
    deleted_origin    TEXT,                         -- 'local' | 'remote'
    created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    FOREIGN KEY (task_id)    REFERENCES tasks(id)         ON DELETE SET NULL,
    FOREIGN KEY (account_id) REFERENCES sync_accounts(id) ON DELETE CASCADE
);

-- A remote item maps to at most one live link; a task maps to at most one live
-- link per account. Partial indexes let a fresh link coexist with an old tombstone.
CREATE UNIQUE INDEX ux_link_remote ON sync_links(account_id, remote_id)
    WHERE remote_id IS NOT NULL AND deleted_at IS NULL;
CREATE UNIQUE INDEX ux_link_local  ON sync_links(task_id, account_id)
    WHERE task_id IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX ix_link_dirty  ON sync_links(account_id, sync_state)
    WHERE sync_state != 'synced';
CREATE INDEX ix_link_dedupe ON sync_links(account_id, dedupe_key)
    WHERE dedupe_key IS NOT NULL;

-- Incremental-sync cursor per account (Google Calendar syncToken, Google Tasks
-- updatedMin high-water mark, or an EventKit change-token blob).
CREATE TABLE sync_cursors (
    account_id    TEXT PRIMARY KEY,
    change_token  TEXT,           -- Google Calendar nextSyncToken | EventKit change token (base64)
    updated_high  TEXT,           -- Google Tasks: max(updated) seen -> next updatedMin
    full_sync_at  TEXT,
    last_pull_at  TEXT,
    status        TEXT NOT NULL DEFAULT 'ok',   -- 'ok' | 'needs_full_sync' | 'error'
    FOREIGN KEY (account_id) REFERENCES sync_accounts(id) ON DELETE CASCADE
);
