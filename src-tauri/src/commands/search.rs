use serde::Serialize;
use sqlx::FromRow;
use tauri::State;

use crate::db::Database;
use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub source: String,
    pub source_id: String,
    pub title: String,
    pub snippet: String,
    pub created_at: String,
    pub conversation_id: Option<String>,
}

fn sanitize_query(raw: &str) -> Option<String> {
    let trimmed: String = raw
        .chars()
        .map(|c| if c == '"' { ' ' } else { c })
        .collect();
    let words: Vec<&str> = trimmed
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .collect();
    if words.is_empty() {
        return None;
    }
    Some(format!("\"{}\"*", words.join(" ")))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn search_index(
    db: State<'_, Database>,
    query: String,
    limit: Option<u32>,
) -> AppResult<Vec<SearchHit>> {
    let limit = limit.unwrap_or(40).min(200);
    let Some(fts_query) = sanitize_query(&query) else {
        return Ok(Vec::new());
    };

    let rows: Vec<(String, String, String, String, String, Option<String>)> = sqlx::query_as(
        "SELECT s.source,
                s.source_id,
                s.title,
                snippet(search_index, 3, '<<', '>>', '…', 12) AS snippet,
                s.created_at,
                m.conversation_id
         FROM search_index s
         LEFT JOIN messages m ON s.source = 'message' AND s.source_id = m.id
         WHERE search_index MATCH ?
         ORDER BY s.created_at DESC
         LIMIT ?",
    )
    .bind(&fts_query)
    .bind(limit)
    .fetch_all(db.pool())
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(source, source_id, title, snippet, created_at, conversation_id)| SearchHit {
                source,
                source_id,
                title,
                snippet,
                created_at,
                conversation_id,
            },
        )
        .collect())
}
