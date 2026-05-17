use chrono::Utc;
use tauri::State;
use uuid::Uuid;

use crate::db::models::{Session, SessionWithTasks, Task};
use crate::db::Database;
use crate::error::{AppError, AppResult};

#[tauri::command(rename_all = "camelCase")]
pub async fn create_session(
    db: State<'_, Database>,
    raw_input: String,
    language: String,
) -> AppResult<Session> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO sessions (id, raw_input, language, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&raw_input)
    .bind(&language)
    .bind(now)
    .execute(db.pool())
    .await?;

    sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE id = ?")
        .bind(&id)
        .fetch_one(db.pool())
        .await
        .map_err(Into::into)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn finalize_session(
    db: State<'_, Database>,
    session_id: String,
    ai_summary: String,
    provider_id: String,
    model: String,
) -> AppResult<Session> {
    let now = Utc::now();

    let rows = sqlx::query(
        "UPDATE sessions
         SET ai_summary = ?, provider_id = ?, model = ?, completed_at = ?
         WHERE id = ?",
    )
    .bind(&ai_summary)
    .bind(&provider_id)
    .bind(&model)
    .bind(now)
    .bind(&session_id)
    .execute(db.pool())
    .await?
    .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound(format!("session {session_id}")));
    }

    sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE id = ?")
        .bind(&session_id)
        .fetch_one(db.pool())
        .await
        .map_err(Into::into)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_session(
    db: State<'_, Database>,
    session_id: String,
) -> AppResult<SessionWithTasks> {
    let session = sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE id = ?")
        .bind(&session_id)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("session {session_id}")))?;

    let tasks = sqlx::query_as::<_, Task>(
        "SELECT * FROM tasks WHERE session_id = ? ORDER BY order_index ASC",
    )
    .bind(&session_id)
    .fetch_all(db.pool())
    .await?;

    Ok(SessionWithTasks { session, tasks })
}

#[tauri::command(rename_all = "camelCase")]
pub async fn list_today_sessions(db: State<'_, Database>) -> AppResult<Vec<SessionWithTasks>> {
    let sessions = sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE date(created_at) = date('now', 'localtime') ORDER BY created_at DESC",
    )
    .fetch_all(db.pool())
    .await?;

    let mut result = Vec::with_capacity(sessions.len());
    for session in sessions {
        let tasks = sqlx::query_as::<_, Task>(
            "SELECT * FROM tasks WHERE session_id = ? ORDER BY order_index ASC",
        )
        .bind(&session.id)
        .fetch_all(db.pool())
        .await?;
        result.push(SessionWithTasks { session, tasks });
    }

    Ok(result)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn list_recent_sessions(
    db: State<'_, Database>,
    limit: i64,
) -> AppResult<Vec<Session>> {
    sqlx::query_as::<_, Session>("SELECT * FROM sessions ORDER BY created_at DESC LIMIT ?")
        .bind(limit.max(1).min(200))
        .fetch_all(db.pool())
        .await
        .map_err(Into::into)
}
