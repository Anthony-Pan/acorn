use chrono::Utc;
use serde::Serialize;
use sqlx::FromRow;
use tauri::State;
use uuid::Uuid;

use crate::db::Database;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Canvas {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command(rename_all = "camelCase")]
pub async fn list_canvases(db: State<'_, Database>) -> AppResult<Vec<Canvas>> {
    sqlx::query_as::<_, Canvas>(
        "SELECT id, title, content, created_at, updated_at
         FROM canvases ORDER BY updated_at DESC LIMIT 100",
    )
    .fetch_all(db.pool())
    .await
    .map_err(Into::into)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_canvas(db: State<'_, Database>, id: String) -> AppResult<Canvas> {
    sqlx::query_as::<_, Canvas>(
        "SELECT id, title, content, created_at, updated_at FROM canvases WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(db.pool())
    .await?
    .ok_or_else(|| AppError::NotFound(format!("canvas {id}")))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn create_canvas(db: State<'_, Database>, title: Option<String>) -> AppResult<Canvas> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let title = title
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| "Untitled canvas".into());
    sqlx::query(
        "INSERT INTO canvases (id, title, content, created_at, updated_at) VALUES (?, ?, '', ?, ?)",
    )
    .bind(&id)
    .bind(&title)
    .bind(&now)
    .bind(&now)
    .execute(db.pool())
    .await?;
    get_canvas(db, id).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_canvas(
    db: State<'_, Database>,
    id: String,
    title: Option<String>,
    content: Option<String>,
) -> AppResult<Canvas> {
    let now = Utc::now().to_rfc3339();
    if let Some(title) = title {
        sqlx::query("UPDATE canvases SET title = ?, updated_at = ? WHERE id = ?")
            .bind(title)
            .bind(&now)
            .bind(&id)
            .execute(db.pool())
            .await?;
    }
    if let Some(content) = content {
        sqlx::query("UPDATE canvases SET content = ?, updated_at = ? WHERE id = ?")
            .bind(content)
            .bind(&now)
            .bind(&id)
            .execute(db.pool())
            .await?;
    }
    get_canvas(db, id).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_canvas(db: State<'_, Database>, id: String) -> AppResult<()> {
    sqlx::query("DELETE FROM canvases WHERE id = ?")
        .bind(&id)
        .execute(db.pool())
        .await?;
    Ok(())
}
