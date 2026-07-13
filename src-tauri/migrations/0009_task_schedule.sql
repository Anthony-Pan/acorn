-- Scheduling + two-way change tracking on tasks (previously lifecycle-only).
--
-- Calendar-event mapping uses scheduled_start / scheduled_end; reminder/task
-- mapping uses due_date. `updated_at` is the last-write-wins clock the sync
-- engine arbitrates conflicts on — every mutating task command bumps it.
ALTER TABLE tasks ADD COLUMN scheduled_start TEXT;
ALTER TABLE tasks ADD COLUMN scheduled_end   TEXT;
ALTER TABLE tasks ADD COLUMN due_date        TEXT;
ALTER TABLE tasks ADD COLUMN all_day         INTEGER NOT NULL DEFAULT 0;
ALTER TABLE tasks ADD COLUMN updated_at      TEXT NOT NULL DEFAULT '1970-01-01T00:00:00Z';

-- SQLite forbids a non-constant DEFAULT on ADD COLUMN, so seed updated_at from
-- created_at for existing rows. New rows set updated_at explicitly in insert_task,
-- so no live row keeps the sentinel.
UPDATE tasks SET updated_at = created_at WHERE updated_at = '1970-01-01T00:00:00Z';
