//! The sync engine. Each account cycle is **push then pull**: local changes go
//! out first (outbox drain over `sync_links`), then the provider's change feed
//! is folded back in — echo-suppressed via `content_hash`, conflicts decided by
//! [`resolve`], remote deletes propagated with the delete-vs-edit guard, and
//! edits fanned out to the task's other accounts. `acorn.db` is the source of
//! truth; providers are peers we reconcile.
//!
//! Pull scope: **Acorn-managed items only.** Changes to remote items Acorn
//! never pushed (no matching `sync_links.remote_id`) are ignored — importing
//! foreign calendars/lists into the stash is a separate product feature.

use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use crate::db::models::Task;
use crate::db::Database;

use super::error::{SyncError, SyncResult};
use super::mapping::{content_hash, map_task};
use super::provider::{build_provider, SyncProvider};
use super::resolve::{resolve, ConflictPolicy, LinkFacts, RemoteFacts, Resolution};
use super::types::{
    PushResult, RemoteChange, SyncAccount, SyncLink, SyncOp, SyncProviderKind, TargetKind,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct PushStats {
    pub created: u32,
    pub updated: u32,
    pub deleted: u32,
    pub failed: u32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PullStats {
    /// Remote edits applied to local tasks.
    pub applied: u32,
    /// Remote deletes propagated locally.
    pub deleted: u32,
    /// Both-changed situations a policy had to decide.
    pub conflicts: u32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CycleStats {
    pub push: PushStats,
    /// `None` when the provider has no pull path yet.
    pub pull: Option<PullStats>,
}

/// Sync every enabled account, best-effort. Returns a per-account result and
/// records `last_synced_at` / `last_error` on each account row.
pub async fn sync_all_enabled(db: &Database) -> Vec<(String, SyncResult<CycleStats>)> {
    let accounts = match load_enabled_accounts(db).await {
        Ok(accounts) => accounts,
        Err(_) => return Vec::new(),
    };
    let mut results = Vec::with_capacity(accounts.len());
    for account in accounts {
        let result = sync_account(db, &account).await;
        record_result(db, &account.id, &result).await;
        results.push((account.id.clone(), result));
    }
    results
}

/// One full cycle for one account: reconcile links, push, then pull.
pub async fn sync_account(db: &Database, account: &SyncAccount) -> SyncResult<CycleStats> {
    ensure_links(db, account).await?;
    let provider = build_provider(account).await?;
    let push = push_with(db, account, provider.as_ref()).await?;
    let pull = pull_with(db, account, provider.as_ref()).await?;
    Ok(CycleStats { push, pull })
}

// ---------------------------------------------------------------------------
// Push (outbox drain)
// ---------------------------------------------------------------------------

async fn push_with(
    db: &Database,
    account: &SyncAccount,
    provider: &dyn SyncProvider,
) -> SyncResult<PushStats> {
    let links = load_dirty_links(db, &account.id).await?;
    if links.is_empty() {
        return Ok(PushStats::default());
    }

    let target = account.target();

    let mut ops: Vec<SyncOp> = Vec::new();
    // link_id -> (content_hash pushed, local_rev at push time) for settling.
    let mut meta: HashMap<String, (Option<String>, i64)> = HashMap::new();
    let mut stats = PushStats::default();

    for link in &links {
        if link.sync_state == "pending_delete" {
            match &link.remote_id {
                Some(remote_id) => {
                    ops.push(SyncOp::Delete {
                        link_id: link.id.clone(),
                        remote_id: remote_id.clone(),
                        etag: link.remote_etag.clone(),
                    });
                    meta.insert(link.id.clone(), (None, link.local_rev));
                }
                // Nothing was ever pushed — drop the tombstone outright.
                None => delete_link(db, &link.id).await?,
            }
            continue;
        }

        // Create / update need the task's current content.
        let Some(task_id) = link.task_id.as_deref() else {
            // A non-delete link with no task is orphaned; clean it up.
            delete_link(db, &link.id).await?;
            continue;
        };
        let Some(task) = load_task(db, task_id).await? else {
            continue;
        };
        let item = map_task(&task, target);
        let hash = content_hash(&item);

        // Skip a redundant update whose content already matches the remote.
        if link.sync_state == "pending_update"
            && link.remote_id.is_some()
            && link.content_hash.as_deref() == Some(hash.as_str())
        {
            settle_no_change(db, &link.id, link.local_rev).await?;
            continue;
        }

        match &link.remote_id {
            Some(remote_id) => ops.push(SyncOp::Update {
                link_id: link.id.clone(),
                remote_id: remote_id.clone(),
                etag: link.remote_etag.clone(),
                item,
            }),
            None => ops.push(SyncOp::Create {
                link_id: link.id.clone(),
                dedupe_key: link.dedupe_key.clone().unwrap_or_else(|| link.id.clone()),
                item,
            }),
        }
        meta.insert(link.id.clone(), (Some(hash), link.local_rev));
    }

    if ops.is_empty() {
        return Ok(stats);
    }

    let outcomes = provider.push(ops).await?;
    for outcome in outcomes {
        let (hash, pushed_rev) = meta.get(&outcome.link_id).cloned().unwrap_or((None, 0));
        apply_push_result(
            db,
            &outcome.link_id,
            outcome.result,
            hash,
            pushed_rev,
            &mut stats,
        )
        .await?;
    }
    Ok(stats)
}

/// Persist the outcome of a single push op. `synced_local_rev` is set to the
/// rev we actually pushed, and the row stays `pending_update` if the user edited
/// the task again while the push was in flight (`local_rev > pushed_rev`).
async fn apply_push_result(
    db: &Database,
    link_id: &str,
    result: PushResult,
    hash: Option<String>,
    pushed_rev: i64,
    stats: &mut PushStats,
) -> SyncResult<()> {
    let now = Utc::now();
    match result {
        PushResult::Created {
            remote_id,
            etag,
            updated,
        } => {
            stats.created += 1;
            sqlx::query(
                "UPDATE sync_links
                    SET remote_id = ?, remote_etag = ?, remote_updated = ?, content_hash = ?,
                        synced_local_rev = ?, last_synced_at = ?, last_error = NULL, retry_count = 0,
                        sync_state = CASE WHEN local_rev > ? THEN 'pending_update' ELSE 'synced' END
                  WHERE id = ?",
            )
            .bind(remote_id)
            .bind(etag)
            .bind(updated)
            .bind(hash)
            .bind(pushed_rev)
            .bind(now)
            .bind(pushed_rev)
            .bind(link_id)
            .execute(db.pool())
            .await?;
        }
        PushResult::Updated { etag, updated } => {
            stats.updated += 1;
            sqlx::query(
                "UPDATE sync_links
                    SET remote_etag = ?, remote_updated = ?, content_hash = ?,
                        synced_local_rev = ?, last_synced_at = ?, last_error = NULL, retry_count = 0,
                        sync_state = CASE WHEN local_rev > ? THEN 'pending_update' ELSE 'synced' END
                  WHERE id = ?",
            )
            .bind(etag)
            .bind(updated)
            .bind(hash)
            .bind(pushed_rev)
            .bind(now)
            .bind(pushed_rev)
            .bind(link_id)
            .execute(db.pool())
            .await?;
        }
        PushResult::Deleted => {
            stats.deleted += 1;
            delete_link(db, link_id).await?;
        }
        PushResult::Conflict => {
            sqlx::query(
                "UPDATE sync_links SET sync_state = 'conflict', last_error = ?, retry_count = retry_count + 1 WHERE id = ?",
            )
            .bind("remote version conflict")
            .bind(link_id)
            .execute(db.pool())
            .await?;
        }
        PushResult::Failed { message } => {
            stats.failed += 1;
            sqlx::query(
                "UPDATE sync_links SET last_error = ?, retry_count = retry_count + 1 WHERE id = ?",
            )
            .bind(message)
            .bind(link_id)
            .execute(db.pool())
            .await?;
        }
    }
    Ok(())
}

/// Settle a link whose content already matches the remote, without a network
/// call. Stays dirty if the task changed again after we read it.
async fn settle_no_change(db: &Database, link_id: &str, pushed_rev: i64) -> SyncResult<()> {
    sqlx::query(
        "UPDATE sync_links
            SET synced_local_rev = ?, last_error = NULL, retry_count = 0,
                sync_state = CASE WHEN local_rev > ? THEN 'pending_update' ELSE 'synced' END
          WHERE id = ?",
    )
    .bind(pushed_rev)
    .bind(pushed_rev)
    .bind(link_id)
    .execute(db.pool())
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Pull (remote change feed → local)
// ---------------------------------------------------------------------------

async fn pull_with(
    db: &Database,
    account: &SyncAccount,
    provider: &dyn SyncProvider,
) -> SyncResult<Option<PullStats>> {
    let links = load_remote_links(db, &account.id).await?;
    let linked_ids: Vec<String> = links.keys().cloned().collect();
    let cursor = load_cursor(db, account).await?;

    let batch = match provider.pull(cursor, linked_ids.clone()).await {
        Ok(batch) => batch,
        // Provider has no pull path yet — skip quietly, push-only cycle.
        Err(SyncError::NotImplemented(_)) => return Ok(None),
        // Cursor rejected (410): clear it and reconcile from a full pull.
        Err(SyncError::FullResyncRequired) => {
            clear_cursor(db, &account.id).await?;
            provider.full_pull(linked_ids).await?
        }
        Err(other) => return Err(other),
    };

    let policy = ConflictPolicy::parse(&account.conflict_policy);
    let target = account.target();
    let mut stats = PullStats::default();

    for change in batch.changes {
        let Some(link) = links.get(&change.remote_id) else {
            // Foreign remote item — not Acorn-managed; out of scope.
            continue;
        };
        // A locally tombstoned link means the pending remote delete owns this
        // item; the push path will settle it.
        if link.deleted_at.is_some() || link.sync_state == "pending_delete" {
            continue;
        }
        apply_remote_change(db, account, target, policy, link, &change, &mut stats).await?;
    }

    store_cursor(db, account, batch.next_cursor).await?;
    Ok(Some(stats))
}

/// Fold one pulled change into the local store, per the [`resolve`] decision.
async fn apply_remote_change(
    db: &Database,
    account: &SyncAccount,
    target: TargetKind,
    policy: ConflictPolicy,
    link: &SyncLink,
    change: &RemoteChange,
    stats: &mut PullStats,
) -> SyncResult<()> {
    let Some(task_id) = link.task_id.as_deref() else {
        return Ok(());
    };
    let Some(task) = load_task(db, task_id).await? else {
        return Ok(());
    };

    let incoming_hash = change.item.as_ref().map(content_hash);
    let local_dirty = link.local_rev > link.synced_local_rev
        || matches!(
            link.sync_state.as_str(),
            "pending_create" | "pending_update"
        );
    let decision = resolve(
        LinkFacts {
            local_dirty,
            synced_hash: link.content_hash.as_deref(),
            local_updated: task.updated_at,
        },
        RemoteFacts {
            deleted: change.deleted,
            incoming_hash: incoming_hash.as_deref(),
            remote_updated: change.updated,
        },
        policy,
    );
    if local_dirty && !matches!(decision, Resolution::Echo) {
        stats.conflicts += 1;
    }

    let now = Utc::now();
    match decision {
        Resolution::Echo => {
            // Our own write coming back — refresh the remote bookkeeping only.
            sqlx::query(
                "UPDATE sync_links SET remote_etag = ?, remote_updated = ?, last_synced_at = ? WHERE id = ?",
            )
            .bind(&change.etag)
            .bind(change.updated)
            .bind(now)
            .bind(&link.id)
            .execute(db.pool())
            .await?;
        }
        Resolution::KeepLocal => {
            // Local wins; adopt the fresh etag so our overwrite push doesn't 412.
            sqlx::query("UPDATE sync_links SET remote_etag = ?, remote_updated = ? WHERE id = ?")
                .bind(&change.etag)
                .bind(change.updated)
                .bind(&link.id)
                .execute(db.pool())
                .await?;
        }
        Resolution::ApplyRemote => {
            let Some(item) = change.item.as_ref() else {
                return Ok(());
            };
            stats.applied += 1;
            let task_updated = change.updated.unwrap_or(now);
            let mut tx = db.pool().begin().await?;

            // Project the remote fields back onto the task. Only the fields
            // this target kind carries are authoritative.
            let (scheduled_start, scheduled_end, due_date, all_day) = match target {
                TargetKind::Event => (item.start, item.end, task.due_date, item.all_day),
                TargetKind::Reminder => (
                    task.scheduled_start,
                    task.scheduled_end,
                    item.due,
                    task.all_day,
                ),
            };
            sqlx::query(
                "UPDATE tasks
                    SET title = ?, description = ?, scheduled_start = ?, scheduled_end = ?,
                        due_date = ?, all_day = ?, updated_at = ?
                  WHERE id = ?",
            )
            .bind(&item.title)
            .bind(&item.notes)
            .bind(scheduled_start)
            .bind(scheduled_end)
            .bind(due_date)
            .bind(all_day)
            .bind(task_updated)
            .bind(task_id)
            .execute(&mut *tx)
            .await?;

            // Done-state: remote completed wins; remote un-complete reopens a
            // completed task but leaves in_progress / skipped alone.
            if item.completed {
                sqlx::query(
                    "UPDATE tasks SET status = 'completed', completed_at = COALESCE(?, completed_at, ?) WHERE id = ?",
                )
                .bind(item.completed_at)
                .bind(now)
                .bind(task_id)
                .execute(&mut *tx)
                .await?;
            } else {
                sqlx::query(
                    "UPDATE tasks SET status = 'pending', completed_at = NULL
                      WHERE id = ? AND status = 'completed'",
                )
                .bind(task_id)
                .execute(&mut *tx)
                .await?;
            }

            // This link is now in sync with the remote we just absorbed…
            let new_hash = incoming_hash.clone();
            sqlx::query(
                "UPDATE sync_links
                    SET remote_etag = ?, remote_updated = ?, content_hash = ?,
                        synced_local_rev = local_rev, sync_state = 'synced', last_synced_at = ?
                  WHERE id = ?",
            )
            .bind(&change.etag)
            .bind(change.updated)
            .bind(new_hash)
            .bind(now)
            .bind(&link.id)
            .execute(&mut *tx)
            .await?;

            // …and the task's OTHER accounts must now receive the edit.
            dirty_other_links(&mut tx, task_id, &account.id).await?;
            tx.commit().await?;
        }
        Resolution::DeleteLocal => {
            stats.deleted += 1;
            let mut tx = db.pool().begin().await?;
            // Fan the delete out to the task's other accounts (tombstones),
            // drop this link (its remote is already gone), then the task row.
            sqlx::query(
                "DELETE FROM sync_links
                  WHERE task_id = ? AND account_id != ? AND remote_id IS NULL",
            )
            .bind(task_id)
            .bind(&account.id)
            .execute(&mut *tx)
            .await?;
            sqlx::query(
                "UPDATE sync_links
                    SET sync_state = 'pending_delete', deleted_origin = 'remote',
                        deleted_at = ?, local_rev = local_rev + 1
                  WHERE task_id = ? AND account_id != ? AND deleted_at IS NULL",
            )
            .bind(now)
            .bind(task_id)
            .bind(&account.id)
            .execute(&mut *tx)
            .await?;
            sqlx::query("DELETE FROM sync_links WHERE id = ?")
                .bind(&link.id)
                .execute(&mut *tx)
                .await?;
            sqlx::query("DELETE FROM tasks WHERE id = ?")
                .bind(task_id)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
        }
        Resolution::ResurrectRemote => {
            // Remote deleted an item the user has unpushed edits on: never
            // drop the edit — recreate it remotely on the next push.
            sqlx::query(
                "UPDATE sync_links
                    SET remote_id = NULL, remote_etag = NULL, remote_updated = NULL,
                        content_hash = NULL, sync_state = 'pending_create'
                  WHERE id = ?",
            )
            .bind(&link.id)
            .execute(db.pool())
            .await?;
        }
    }
    Ok(())
}

/// Mark every live link of `task_id` on accounts other than `origin` dirty, so
/// an absorbed remote edit propagates across providers.
async fn dirty_other_links(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    task_id: &str,
    origin_account_id: &str,
) -> SyncResult<()> {
    sqlx::query(
        "UPDATE sync_links
            SET local_rev = local_rev + 1,
                sync_state = CASE WHEN remote_id IS NULL THEN 'pending_create' ELSE 'pending_update' END
          WHERE task_id = ? AND account_id != ? AND deleted_at IS NULL",
    )
    .bind(task_id)
    .bind(origin_account_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Cursors
// ---------------------------------------------------------------------------

/// Google Tasks cursors live in `updated_high` (an RFC3339 high-water mark);
/// everything else uses the opaque `change_token`.
fn cursor_column(account: &SyncAccount) -> &'static str {
    match account.kind() {
        Some(SyncProviderKind::GoogleTasks) => "updated_high",
        _ => "change_token",
    }
}

async fn load_cursor(db: &Database, account: &SyncAccount) -> SyncResult<Option<String>> {
    let column = cursor_column(account);
    let row: Option<(Option<String>,)> = sqlx::query_as(&format!(
        "SELECT {column} FROM sync_cursors WHERE account_id = ?"
    ))
    .bind(&account.id)
    .fetch_optional(db.pool())
    .await?;
    Ok(row.and_then(|(cursor,)| cursor))
}

async fn store_cursor(
    db: &Database,
    account: &SyncAccount,
    next: Option<String>,
) -> SyncResult<()> {
    let column = cursor_column(account);
    let now = Utc::now();
    sqlx::query(&format!(
        "INSERT INTO sync_cursors (account_id, {column}, last_pull_at, status)
         VALUES (?, ?, ?, 'ok')
         ON CONFLICT(account_id) DO UPDATE SET
             {column} = COALESCE(excluded.{column}, {column}),
             last_pull_at = excluded.last_pull_at,
             status = 'ok'"
    ))
    .bind(&account.id)
    .bind(next)
    .bind(now)
    .execute(db.pool())
    .await?;
    Ok(())
}

async fn clear_cursor(db: &Database, account_id: &str) -> SyncResult<()> {
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO sync_cursors (account_id, change_token, updated_high, full_sync_at, status)
         VALUES (?, NULL, NULL, ?, 'needs_full_sync')
         ON CONFLICT(account_id) DO UPDATE SET
             change_token = NULL, updated_high = NULL,
             full_sync_at = excluded.full_sync_at, status = 'needs_full_sync'",
    )
    .bind(account_id)
    .bind(now)
    .execute(db.pool())
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Shared queries
// ---------------------------------------------------------------------------

/// Create `pending_create` links for tasks that should sync to this account but
/// don't have a live link yet. Scoped to active or recent tasks so connecting an
/// account doesn't flood a calendar with the entire task history.
async fn ensure_links(db: &Database, account: &SyncAccount) -> SyncResult<()> {
    let task_ids: Vec<(String,)> = sqlx::query_as(
        "SELECT t.id FROM tasks t
          WHERE (t.status IN ('pending', 'in_progress')
                 OR date(t.created_at) >= date('now', '-30 days'))
            AND NOT EXISTS (
                SELECT 1 FROM sync_links l
                 WHERE l.task_id = t.id AND l.account_id = ? AND l.deleted_at IS NULL
            )",
    )
    .bind(&account.id)
    .fetch_all(db.pool())
    .await?;

    for (task_id,) in task_ids {
        let link_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO sync_links (id, task_id, account_id, dedupe_key, sync_state)
             VALUES (?, ?, ?, ?, 'pending_create')",
        )
        .bind(&link_id)
        .bind(&task_id)
        .bind(&account.id)
        .bind(&link_id)
        .execute(db.pool())
        .await?;
    }
    Ok(())
}

async fn load_dirty_links(db: &Database, account_id: &str) -> SyncResult<Vec<SyncLink>> {
    sqlx::query_as::<_, SyncLink>(
        "SELECT * FROM sync_links
          WHERE account_id = ?
            AND sync_state IN ('pending_create', 'pending_update', 'pending_delete')
          ORDER BY created_at ASC",
    )
    .bind(account_id)
    .fetch_all(db.pool())
    .await
    .map_err(Into::into)
}

/// Live, remote-backed links for an account, keyed by `remote_id`.
async fn load_remote_links(
    db: &Database,
    account_id: &str,
) -> SyncResult<HashMap<String, SyncLink>> {
    let links = sqlx::query_as::<_, SyncLink>(
        "SELECT * FROM sync_links
          WHERE account_id = ? AND remote_id IS NOT NULL AND deleted_at IS NULL",
    )
    .bind(account_id)
    .fetch_all(db.pool())
    .await?;
    Ok(links
        .into_iter()
        .filter_map(|link| link.remote_id.clone().map(|remote_id| (remote_id, link)))
        .collect())
}

async fn load_task(db: &Database, task_id: &str) -> SyncResult<Option<Task>> {
    sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?")
        .bind(task_id)
        .fetch_optional(db.pool())
        .await
        .map_err(Into::into)
}

async fn load_enabled_accounts(db: &Database) -> SyncResult<Vec<SyncAccount>> {
    sqlx::query_as::<_, SyncAccount>("SELECT * FROM sync_accounts WHERE enabled = 1")
        .fetch_all(db.pool())
        .await
        .map_err(Into::into)
}

async fn delete_link(db: &Database, link_id: &str) -> SyncResult<()> {
    sqlx::query("DELETE FROM sync_links WHERE id = ?")
        .bind(link_id)
        .execute(db.pool())
        .await?;
    Ok(())
}

/// Record a sync attempt's outcome on the account row (`last_synced_at` on
/// success, `last_error` on failure). Public so `trigger_sync_now` can reuse it.
pub async fn record_result(db: &Database, account_id: &str, result: &SyncResult<CycleStats>) {
    let now = Utc::now();
    let query = match result {
        Ok(_) => sqlx::query(
            "UPDATE sync_accounts SET last_synced_at = ?, last_error = NULL WHERE id = ?",
        )
        .bind(now)
        .bind(account_id),
        Err(err) => sqlx::query("UPDATE sync_accounts SET last_error = ? WHERE id = ?")
            .bind(err.to_string())
            .bind(account_id),
    };
    let _ = query.execute(db.pool()).await;
}
