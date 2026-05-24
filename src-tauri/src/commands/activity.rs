use chrono::Utc;
use serde::Serialize;
use sqlx::FromRow;
use tauri::State;
use uuid::Uuid;

use crate::db::Database;
use crate::error::AppResult;

const SENSITIVE_PATTERNS: &[&str] = &[
    "password",
    "passwd",
    "credential",
    "secret",
    "api_key",
    "apikey",
    "token=",
    ".env",
    "salary",
    "薪资",
    "合同",
    "密码",
    "信用卡",
];

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ActivityEntry {
    pub id: String,
    pub kind: String,
    pub content: String,
    pub created_at: String,
}

fn is_sensitive(content: &str) -> bool {
    let lower = content.to_lowercase();
    SENSITIVE_PATTERNS.iter().any(|p| lower.contains(p))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn record_activity(
    db: State<'_, Database>,
    kind: String,
    content: String,
) -> AppResult<Option<ActivityEntry>> {
    let trimmed = content.trim();
    if trimmed.is_empty() || is_sensitive(trimmed) {
        return Ok(None);
    }
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let truncated = if trimmed.len() > 500 {
        format!("{}…", &trimmed[..500])
    } else {
        trimmed.to_string()
    };
    sqlx::query("INSERT INTO activity_log (id, kind, content, created_at) VALUES (?, ?, ?, ?)")
        .bind(&id)
        .bind(&kind)
        .bind(&truncated)
        .bind(&now)
        .execute(db.pool())
        .await?;
    Ok(Some(ActivityEntry {
        id,
        kind,
        content: truncated,
        created_at: now,
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn list_recent_activity(
    db: State<'_, Database>,
    limit: Option<u32>,
) -> AppResult<Vec<ActivityEntry>> {
    let limit = limit.unwrap_or(50).min(500);
    sqlx::query_as::<_, ActivityEntry>(
        "SELECT id, kind, content, created_at FROM activity_log ORDER BY created_at DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(db.pool())
    .await
    .map_err(Into::into)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn clear_activity(db: State<'_, Database>) -> AppResult<()> {
    sqlx::query("DELETE FROM activity_log")
        .execute(db.pool())
        .await?;
    Ok(())
}
