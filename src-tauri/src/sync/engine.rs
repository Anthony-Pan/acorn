//! The sync engine. Phase 1 is the outbox: reconcile which tasks should sync,
//! then push local changes (create/update/delete) out to each provider and fold
//! the results back into `sync_links`. `acorn.db` is the source of truth; the
//! pull + conflict-resolution half of two-way sync layers on in later phases.

use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use crate::db::models::Task;
use crate::db::Database;

use super::error::SyncResult;
use super::mapping::{content_hash, map_task};
use super::provider::build_provider;
use super::types::{PushResult, SyncAccount, SyncLink, SyncOp};

#[derive(Debug, Default, Clone, Copy)]
pub struct PushStats {
    pub created: u32,
    pub updated: u32,
    pub deleted: u32,
    pub failed: u32,
}

/// Sync every enabled account, best-effort. Returns a per-account result and
/// records `last_synced_at` / `last_error` on each account row.
pub async fn sync_all_enabled(db: &Database) -> Vec<(String, SyncResult<PushStats>)> {
    let accounts = match load_enabled_accounts(db).await {
        Ok(accounts) => accounts,
        Err(_) => return Vec::new(),
    };
    let mut results = Vec::with_capacity(accounts.len());
    for account in accounts {
        let result = push_account(db, &account).await;
        record_result(db, &account.id, &result).await;
        results.push((account.id.clone(), result));
    }
    results
}

/// Push one account's pending links. Also runs the reconcile pass that creates
/// `pending_create` links for recent/active tasks that don't yet have one.
pub async fn push_account(db: &Database, account: &SyncAccount) -> SyncResult<PushStats> {
    ensure_links(db, account).await?;

    let links = load_dirty_links(db, &account.id).await?;
    if links.is_empty() {
        return Ok(PushStats::default());
    }

    let provider = build_provider(account).await?;
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
pub async fn record_result(db: &Database, account_id: &str, result: &SyncResult<PushStats>) {
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
