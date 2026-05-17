use chrono::Utc;
use tauri::State;

use crate::db::models::ProviderConfig;
use crate::db::Database;
use crate::error::AppResult;

pub const ACTIVE_PROVIDER_KEY: &str = "active_provider";

#[tauri::command(rename_all = "camelCase")]
pub async fn list_provider_configs(db: State<'_, Database>) -> AppResult<Vec<ProviderConfig>> {
    sqlx::query_as::<_, ProviderConfig>(
        "SELECT * FROM provider_configs ORDER BY last_used_at DESC NULLS LAST, provider_id ASC",
    )
    .fetch_all(db.pool())
    .await
    .map_err(Into::into)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn save_provider_config(
    db: State<'_, Database>,
    provider_id: String,
    enabled: bool,
    custom_endpoint: Option<String>,
    selected_model: Option<String>,
) -> AppResult<ProviderConfig> {
    sqlx::query(
        "INSERT INTO provider_configs (provider_id, enabled, custom_endpoint, selected_model)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(provider_id) DO UPDATE SET
             enabled = excluded.enabled,
             custom_endpoint = excluded.custom_endpoint,
             selected_model = excluded.selected_model",
    )
    .bind(&provider_id)
    .bind(enabled)
    .bind(&custom_endpoint)
    .bind(&selected_model)
    .execute(db.pool())
    .await?;

    sqlx::query_as::<_, ProviderConfig>(
        "SELECT * FROM provider_configs WHERE provider_id = ?",
    )
    .bind(&provider_id)
    .fetch_one(db.pool())
    .await
    .map_err(Into::into)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn touch_provider(
    db: State<'_, Database>,
    provider_id: String,
) -> AppResult<()> {
    let now = Utc::now();
    sqlx::query("UPDATE provider_configs SET last_used_at = ? WHERE provider_id = ?")
        .bind(now)
        .bind(&provider_id)
        .execute(db.pool())
        .await?;
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_active_provider(db: State<'_, Database>) -> AppResult<Option<String>> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT value FROM settings WHERE key = ?")
            .bind(ACTIVE_PROVIDER_KEY)
            .fetch_optional(db.pool())
            .await?;
    Ok(row.map(|(v,)| v))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn set_active_provider(
    db: State<'_, Database>,
    provider_id: String,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(ACTIVE_PROVIDER_KEY)
    .bind(&provider_id)
    .execute(db.pool())
    .await?;
    Ok(())
}
