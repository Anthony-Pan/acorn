use tauri::ipc::Channel;
use tauri::State;
use uuid::Uuid;

use crate::db::Database;
use crate::error::{AppError, AppResult};
use crate::sync::engine;
use crate::sync::keychain;
use crate::sync::provider::build_provider;
use crate::sync::providers::google_oauth;
use crate::sync::scheduler::SyncSignal;
use crate::sync::types::{
    require_kind, RemoteContainer, SyncAccount, SyncEvent, SyncProviderKind, SyncStatus,
};

async fn load_account(db: &Database, account_id: &str) -> AppResult<SyncAccount> {
    sqlx::query_as::<_, SyncAccount>("SELECT * FROM sync_accounts WHERE id = ?")
        .bind(account_id)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("sync account {account_id}")))
}

async fn load_enabled_accounts(db: &Database) -> AppResult<Vec<SyncAccount>> {
    sqlx::query_as::<_, SyncAccount>(
        "SELECT * FROM sync_accounts WHERE enabled = 1 ORDER BY created_at ASC",
    )
    .fetch_all(db.pool())
    .await
    .map_err(Into::into)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn list_sync_accounts(db: State<'_, Database>) -> AppResult<Vec<SyncAccount>> {
    sqlx::query_as::<_, SyncAccount>("SELECT * FROM sync_accounts ORDER BY created_at ASC")
        .fetch_all(db.pool())
        .await
        .map_err(Into::into)
}

/// Connect a Google account (Calendar or Tasks). Runs the OAuth loopback flow,
/// stashes the refresh token in the keychain, and inserts the account row. The
/// target kind is fixed by the provider (calendar → event, tasks → reminder).
#[tauri::command(rename_all = "camelCase")]
pub async fn connect_google_account(
    db: State<'_, Database>,
    signal: State<'_, SyncSignal>,
    provider: String,
) -> AppResult<SyncAccount> {
    let kind = require_kind(&provider)?;
    if !kind.is_google() {
        return Err(AppError::InvalidInput(format!(
            "{provider} is not a Google provider"
        )));
    }

    let tokens = google_oauth::run_oauth_flow(kind).await?;
    let id = Uuid::new_v4().to_string();
    keychain::save_refresh_token(&provider, &id, &tokens.refresh_token)?;

    sqlx::query(
        "INSERT INTO sync_accounts (id, provider, target_kind, account_label, has_token, enabled)
         VALUES (?, ?, ?, ?, 1, 1)",
    )
    .bind(&id)
    .bind(&provider)
    .bind(kind.natural_target().as_str())
    .bind(&tokens.email)
    .execute(db.pool())
    .await?;

    // Kick the first sync so the connection shows results within seconds.
    signal.ping();
    load_account(&db, &id).await
}

/// Connect an Apple account (Calendar or Reminders). macOS-only; the EventKit
/// full-access prompt is requested by the provider on first sync.
#[tauri::command(rename_all = "camelCase")]
pub async fn connect_apple_account(
    db: State<'_, Database>,
    signal: State<'_, SyncSignal>,
    provider: String,
) -> AppResult<SyncAccount> {
    let kind = require_kind(&provider)?;
    if !kind.is_apple() {
        return Err(AppError::InvalidInput(format!(
            "{provider} is not an Apple provider"
        )));
    }
    #[cfg(not(target_os = "macos"))]
    {
        return Err(AppError::Sync(
            "Apple Calendar/Reminders sync requires macOS".into(),
        ));
    }
    #[cfg(target_os = "macos")]
    {
        let id = Uuid::new_v4().to_string();
        let label = match kind {
            SyncProviderKind::AppleCalendar => "Apple Calendar",
            SyncProviderKind::AppleReminders => "Apple Reminders",
            _ => "Apple",
        };
        sqlx::query(
            "INSERT INTO sync_accounts (id, provider, target_kind, account_label, has_token, enabled)
             VALUES (?, ?, ?, ?, 0, 1)",
        )
        .bind(&id)
        .bind(&provider)
        .bind(kind.natural_target().as_str())
        .bind(label)
        .execute(db.pool())
        .await?;
        signal.ping();
        load_account(&db, &id).await
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn disconnect_sync_account(db: State<'_, Database>, account_id: String) -> AppResult<()> {
    let account = load_account(&db, &account_id).await?;
    if account
        .kind()
        .map(SyncProviderKind::is_google)
        .unwrap_or(false)
    {
        keychain::delete_refresh_token(&account.provider, &account.id)?;
    }
    // Cascades sync_links and sync_cursors.
    sqlx::query("DELETE FROM sync_accounts WHERE id = ?")
        .bind(&account_id)
        .execute(db.pool())
        .await?;
    Ok(())
}

/// List the calendars / task lists the account can sync into, so the user can
/// pick one.
#[tauri::command(rename_all = "camelCase")]
pub async fn list_remote_containers(
    db: State<'_, Database>,
    account_id: String,
) -> AppResult<Vec<RemoteContainer>> {
    let account = load_account(&db, &account_id).await?;
    let provider = build_provider(&account).await?;
    Ok(provider.list_containers().await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn set_account_container(
    db: State<'_, Database>,
    signal: State<'_, SyncSignal>,
    account_id: String,
    container_id: String,
    container_name: String,
) -> AppResult<SyncAccount> {
    let rows =
        sqlx::query("UPDATE sync_accounts SET container_id = ?, container_name = ? WHERE id = ?")
            .bind(&container_id)
            .bind(&container_name)
            .bind(&account_id)
            .execute(db.pool())
            .await?
            .rows_affected();
    if rows == 0 {
        return Err(AppError::NotFound(format!("sync account {account_id}")));
    }
    signal.ping();
    load_account(&db, &account_id).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn set_account_enabled(
    db: State<'_, Database>,
    signal: State<'_, SyncSignal>,
    account_id: String,
    enabled: bool,
) -> AppResult<SyncAccount> {
    let rows = sqlx::query("UPDATE sync_accounts SET enabled = ? WHERE id = ?")
        .bind(enabled)
        .bind(&account_id)
        .execute(db.pool())
        .await?
        .rows_affected();
    if rows == 0 {
        return Err(AppError::NotFound(format!("sync account {account_id}")));
    }
    if enabled {
        signal.ping();
    }
    load_account(&db, &account_id).await
}

/// Run a sync cycle now for one account (or all enabled), streaming progress
/// over a `Channel<SyncEvent>`, mirroring `decompose`.
#[tauri::command(rename_all = "camelCase")]
pub async fn trigger_sync_now(
    db: State<'_, Database>,
    account_id: Option<String>,
    on_event: Channel<SyncEvent>,
) -> AppResult<()> {
    let accounts = match account_id {
        Some(id) => vec![load_account(&db, &id).await?],
        None => load_enabled_accounts(&db).await?,
    };

    for account in accounts {
        let _ = on_event.send(SyncEvent::Started {
            account_id: account.id.clone(),
            account_label: account.account_label.clone(),
        });
        let result = engine::sync_account(&db, &account).await;
        engine::record_result(&db, &account.id, &result).await;
        match result {
            Ok(stats) => {
                let _ = on_event.send(SyncEvent::Pushed {
                    account_id: account.id.clone(),
                    created: stats.push.created,
                    updated: stats.push.updated,
                    deleted: stats.push.deleted,
                    failed: stats.push.failed,
                });
                if let Some(pull) = stats.pull {
                    let _ = on_event.send(SyncEvent::Pulled {
                        account_id: account.id.clone(),
                        applied: pull.applied,
                        deleted: pull.deleted,
                        conflicts: pull.conflicts,
                    });
                }
                let _ = on_event.send(SyncEvent::Finished {
                    account_id: account.id.clone(),
                });
            }
            Err(err) => {
                let _ = on_event.send(SyncEvent::Error {
                    account_id: account.id.clone(),
                    message: err.to_string(),
                });
            }
        }
    }
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_sync_status(
    db: State<'_, Database>,
    account_id: Option<String>,
) -> AppResult<Vec<SyncStatus>> {
    let accounts = match account_id {
        Some(id) => vec![load_account(&db, &id).await?],
        None => {
            sqlx::query_as::<_, SyncAccount>("SELECT * FROM sync_accounts ORDER BY created_at ASC")
                .fetch_all(db.pool())
                .await?
        }
    };

    let mut out = Vec::with_capacity(accounts.len());
    for account in accounts {
        let (pending, conflicts): (i64, i64) = sqlx::query_as(
            "SELECT
                count(*) FILTER (WHERE sync_state IN ('pending_create', 'pending_update', 'pending_delete')),
                count(*) FILTER (WHERE sync_state = 'conflict')
              FROM sync_links WHERE account_id = ?",
        )
        .bind(&account.id)
        .fetch_one(db.pool())
        .await?;
        out.push(SyncStatus {
            account_id: account.id,
            enabled: account.enabled,
            last_synced_at: account.last_synced_at,
            last_error: account.last_error,
            pending,
            conflicts,
        });
    }
    Ok(out)
}
