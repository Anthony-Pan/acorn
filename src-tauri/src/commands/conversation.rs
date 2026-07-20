use chrono::Utc;
use serde::Serialize;
use sqlx::FromRow;
use tauri::State;
use uuid::Uuid;

use crate::db::models::{Conversation, ConversationWithMessages, Message};
use crate::db::Database;
use crate::error::{AppError, AppResult};

/// A conversation summarized for the knowledge-cloud starfield: enough to size,
/// place, and label a star without loading message bodies.
#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CloudNode {
    pub id: String,
    pub title: String,
    pub last_message_at: String,
    pub message_count: i64,
    pub favorite: bool,
    pub archived: bool,
}

#[tauri::command(rename_all = "camelCase")]
pub async fn list_conversation_cloud(
    db: State<'_, Database>,
    include_archived: bool,
) -> AppResult<Vec<CloudNode>> {
    sqlx::query_as::<_, CloudNode>(
        "SELECT c.id, c.title, c.last_message_at,
                (SELECT COUNT(*) FROM messages m WHERE m.conversation_id = c.id) AS message_count,
                c.favorite AS favorite,
                (c.archived_at IS NOT NULL) AS archived
         FROM conversations c
         WHERE (? OR c.archived_at IS NULL)
         ORDER BY c.last_message_at DESC
         LIMIT 500",
    )
    .bind(include_archived)
    .fetch_all(db.pool())
    .await
    .map_err(Into::into)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn set_conversation_favorite(
    db: State<'_, Database>,
    conversation_id: String,
    favorite: bool,
) -> AppResult<()> {
    sqlx::query("UPDATE conversations SET favorite = ? WHERE id = ?")
        .bind(favorite)
        .bind(&conversation_id)
        .execute(db.pool())
        .await?;
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn set_conversation_archived(
    db: State<'_, Database>,
    conversation_id: String,
    archived: bool,
) -> AppResult<()> {
    let archived_at = if archived {
        Some(Utc::now().to_rfc3339())
    } else {
        None
    };
    sqlx::query("UPDATE conversations SET archived_at = ? WHERE id = ?")
        .bind(archived_at)
        .bind(&conversation_id)
        .execute(db.pool())
        .await?;
    Ok(())
}

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

#[tauri::command(rename_all = "camelCase")]
pub async fn set_conversation_provider(
    db: State<'_, Database>,
    conversation_id: String,
    provider_id: String,
    model: Option<String>,
) -> AppResult<Conversation> {
    sqlx::query("UPDATE conversations SET provider_id = ?, model = ? WHERE id = ?")
        .bind(&provider_id)
        .bind(model.as_deref())
        .bind(&conversation_id)
        .execute(db.pool())
        .await?;
    sqlx::query_as::<_, Conversation>("SELECT * FROM conversations WHERE id = ?")
        .bind(&conversation_id)
        .fetch_one(db.pool())
        .await
        .map_err(Into::into)
}
