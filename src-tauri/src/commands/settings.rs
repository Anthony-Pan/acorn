use tauri::State;

use crate::db::Database;
use crate::error::AppResult;

/// Read a single setting value directly from a pool reference — used at startup
/// (before the managed `State` is available to commands).
pub async fn load_setting(db: &Database, key: &str) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(db.pool())
        .await?;
    Ok(row.map(|(v,)| v))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_setting(db: State<'_, Database>, key: String) -> AppResult<Option<String>> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
        .bind(&key)
        .fetch_optional(db.pool())
        .await?;
    Ok(row.map(|(v,)| v))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn set_setting(db: State<'_, Database>, key: String, value: String) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(&key)
    .bind(&value)
    .execute(db.pool())
    .await?;
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_setting(db: State<'_, Database>, key: String) -> AppResult<()> {
    sqlx::query("DELETE FROM settings WHERE key = ?")
        .bind(&key)
        .execute(db.pool())
        .await?;
    Ok(())
}
