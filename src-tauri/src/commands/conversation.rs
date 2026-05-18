use chrono::Utc;
use tauri::State;
use uuid::Uuid;

use crate::db::models::{Conversation, ConversationWithMessages, Message};
use crate::db::Database;
use crate::error::{AppError, AppResult};

#[tauri::command(rename_all = "camelCase")]
pub async fn create_conversation(
    db: State<'_, Database>,
    title: Option<String>,
) -> AppResult<Conversation> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let title = title.unwrap_or_else(|| "Untitled".to_string());

    sqlx::query(
        "INSERT INTO conversations (id, title, created_at, last_message_at)
         VALUES (?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&title)
    .bind(now)
    .bind(now)
    .execute(db.pool())
    .await?;

    sqlx::query_as::<_, Conversation>("SELECT * FROM conversations WHERE id = ?")
        .bind(&id)
        .fetch_one(db.pool())
        .await
        .map_err(Into::into)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn list_conversations(db: State<'_, Database>) -> AppResult<Vec<Conversation>> {
    sqlx::query_as::<_, Conversation>(
        "SELECT * FROM conversations ORDER BY last_message_at DESC LIMIT 100",
    )
    .fetch_all(db.pool())
    .await
    .map_err(Into::into)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_conversation(
    db: State<'_, Database>,
    conversation_id: String,
) -> AppResult<ConversationWithMessages> {
    let conversation =
        sqlx::query_as::<_, Conversation>("SELECT * FROM conversations WHERE id = ?")
            .bind(&conversation_id)
            .fetch_optional(db.pool())
            .await?
            .ok_or_else(|| AppError::NotFound(format!("conversation {conversation_id}")))?;

    let messages = sqlx::query_as::<_, Message>(
        "SELECT * FROM messages WHERE conversation_id = ? ORDER BY created_at ASC",
    )
    .bind(&conversation_id)
    .fetch_all(db.pool())
    .await?;

    Ok(ConversationWithMessages {
        conversation,
        messages,
    })
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_conversation(
    db: State<'_, Database>,
    conversation_id: String,
) -> AppResult<()> {
    sqlx::query("DELETE FROM conversations WHERE id = ?")
        .bind(&conversation_id)
        .execute(db.pool())
        .await?;
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn rename_conversation(
    db: State<'_, Database>,
    conversation_id: String,
    title: String,
) -> AppResult<Conversation> {
    sqlx::query("UPDATE conversations SET title = ? WHERE id = ?")
        .bind(&title)
        .bind(&conversation_id)
        .execute(db.pool())
        .await?;
    sqlx::query_as::<_, Conversation>("SELECT * FROM conversations WHERE id = ?")
        .bind(&conversation_id)
        .fetch_one(db.pool())
        .await
        .map_err(Into::into)
}
