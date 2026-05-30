use chrono::Utc;
use serde::Serialize;
use sqlx::FromRow;
use tauri::State;
use uuid::Uuid;

use crate::db::Database;
use crate::error::{AppError, AppResult};

/// A pinned reply rendered as an independent always-on-top desktop card.
/// `x`/`y` are the card's last logical-pixel position (NULL until first moved,
/// in which case it is opened with a cascade).
#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Pin {
    pub id: String,
    pub conversation_id: Option<String>,
    pub message_id: Option<String>,
    pub label: String,
    pub content: String,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub created_at: String,
}

const SELECT_COLUMNS: &str = "id, conversation_id, message_id, label, content, x, y, created_at";

/// All pins, oldest first, so restored windows cascade in creation order.
pub async fn load_pins(db: &Database) -> Result<Vec<Pin>, sqlx::Error> {
    sqlx::query_as::<_, Pin>(&format!(
        "SELECT {SELECT_COLUMNS} FROM pins ORDER BY created_at ASC LIMIT 100"
    ))
    .fetch_all(db.pool())
    .await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn list_pins(db: State<'_, Database>) -> AppResult<Vec<Pin>> {
    load_pins(&db).await.map_err(Into::into)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_pin(db: State<'_, Database>, id: String) -> AppResult<Pin> {
    sqlx::query_as::<_, Pin>(&format!("SELECT {SELECT_COLUMNS} FROM pins WHERE id = ?"))
        .bind(&id)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("pin {id}")))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn create_pin(
    db: State<'_, Database>,
    conversation_id: Option<String>,
    message_id: Option<String>,
    label: Option<String>,
    content: String,
) -> AppResult<Pin> {
    let content = content.trim().to_string();
    if content.is_empty() {
        return Err(AppError::InvalidInput("cannot pin empty content".into()));
    }
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let label = label
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .unwrap_or_else(|| "Pinned".into());
    sqlx::query(
        "INSERT INTO pins (id, conversation_id, message_id, label, content, x, y, created_at)
         VALUES (?, ?, ?, ?, ?, NULL, NULL, ?)",
    )
    .bind(&id)
    .bind(&conversation_id)
    .bind(&message_id)
    .bind(&label)
    .bind(&content)
    .bind(&now)
    .execute(db.pool())
    .await?;
    get_pin(db, id).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_pin_position(
    db: State<'_, Database>,
    id: String,
    x: f64,
    y: f64,
) -> AppResult<()> {
    sqlx::query("UPDATE pins SET x = ?, y = ? WHERE id = ?")
        .bind(x)
        .bind(y)
        .bind(&id)
        .execute(db.pool())
        .await?;
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_pin(db: State<'_, Database>, id: String) -> AppResult<()> {
    sqlx::query("DELETE FROM pins WHERE id = ?")
        .bind(&id)
        .execute(db.pool())
        .await?;
    Ok(())
}
