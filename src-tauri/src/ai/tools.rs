use chrono::Utc;
use serde::Serialize;
use sqlx::SqlitePool;
use uuid::Uuid;

use super::error::{ProviderError, ProviderResult};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: serde_json::Value,
}

pub fn tool_catalog() -> Vec<ToolSpec> {
    vec![
        ToolSpec {
            name: "list_today_tasks",
            description: "List the tasks in today's most recent stash. Returns each task's id, \
                title, priority, status, and duration. Use this before answering questions like \
                'what am I working on?' or 'how am I doing?'.",
            parameters: serde_json::json!({ "type": "object", "properties": {} }),
        },
        ToolSpec {
            name: "add_task",
            description: "Append a new task to today's most recent stash session. If there is no \
                session yet, one is created. Use when the user asks you to remember something \
                they need to do (e.g. 'remind me to call mom at 3pm').",
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "title": { "type": "string", "description": "Short, actionable task title (5-15 words)." },
                    "priority": { "type": "string", "enum": ["high", "medium", "low"], "default": "medium" },
                    "duration_minutes": { "type": "integer", "minimum": 1, "default": 15 },
                    "description": { "type": "string", "description": "Optional short context for the task." }
                },
                "required": ["title"]
            }),
        },
        ToolSpec {
            name: "start_task",
            description: "Mark a task as in_progress. Use when the user says they're starting \
                something. Only one task can be in progress at a time.",
            parameters: serde_json::json!({
                "type": "object",
                "properties": { "task_id": { "type": "string" } },
                "required": ["task_id"]
            }),
        },
        ToolSpec {
            name: "complete_task",
            description:
                "Mark a task as completed. Use when the user says they finished something.",
            parameters: serde_json::json!({
                "type": "object",
                "properties": { "task_id": { "type": "string" } },
                "required": ["task_id"]
            }),
        },
        ToolSpec {
            name: "skip_task",
            description: "Mark a task as skipped — the user no longer plans to do it today.",
            parameters: serde_json::json!({
                "type": "object",
                "properties": { "task_id": { "type": "string" } },
                "required": ["task_id"]
            }),
        },
        ToolSpec {
            name: "search_past_activity",
            description: "Full-text search across the user's past chat messages, task titles \
                and descriptions, and stash inputs. Use when the user asks about something \
                they did before ('did I finish the interview prep?', 'what did I work on \
                last Tuesday?'). Returns up to 10 short snippets, each tagged with its source \
                (message / task / session) and id. Do not invent results — only quote what \
                comes back.",
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Free-text search terms." }
                },
                "required": ["query"]
            }),
        },
    ]
}

pub async fn execute_tool(
    pool: &SqlitePool,
    name: &str,
    arguments: &serde_json::Value,
) -> ProviderResult<String> {
    match name {
        "list_today_tasks" => list_today_tasks(pool).await,
        "add_task" => add_task(pool, arguments).await,
        "start_task" => set_status(pool, arguments, "in_progress").await,
        "complete_task" => set_status(pool, arguments, "completed").await,
        "skip_task" => set_status(pool, arguments, "skipped").await,
        "search_past_activity" => search_past_activity(pool, arguments).await,
        other => Err(ProviderError::InvalidResponse(format!("unknown tool {other}"))),
    }
}

async fn search_past_activity(
    pool: &SqlitePool,
    args: &serde_json::Value,
) -> ProviderResult<String> {
    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ProviderError::InvalidResponse("search_past_activity: missing query".into()))?;
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok("[]".to_string());
    }

    let rows: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT source, source_id, created_at, snippet(search_index, 3, '[[', ']]', '...', 12) AS snippet
         FROM search_index
         WHERE search_index MATCH ?
         ORDER BY created_at DESC
         LIMIT 10",
    )
    .bind(trimmed)
    .fetch_all(pool)
    .await
    .map_err(|err| ProviderError::ProviderResponse(format!("search failed: {err}")))?;

    if rows.is_empty() {
        return Ok("[]".to_string());
    }

    let payload: Vec<_> = rows
        .into_iter()
        .map(|(source, source_id, created_at, snippet)| {
            serde_json::json!({
                "source": source,
                "source_id": source_id,
                "created_at": created_at,
                "snippet": snippet,
            })
        })
        .collect();

    Ok(serde_json::to_string(&payload)?)
}

async fn list_today_tasks(pool: &SqlitePool) -> ProviderResult<String> {
    let rows: Vec<(String, String, String, String, i64)> = sqlx::query_as(
        "SELECT t.id, t.title, t.priority, t.status, t.duration_minutes
         FROM tasks t
         JOIN sessions s ON t.session_id = s.id
         WHERE date(s.created_at) = date('now', 'localtime')
         ORDER BY t.order_index ASC",
    )
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Ok("(no tasks today)".to_string());
    }

    let payload = rows
        .into_iter()
        .map(|(id, title, priority, status, duration)| {
            serde_json::json!({
                "id": id,
                "title": title,
                "priority": priority,
                "status": status,
                "duration_minutes": duration,
            })
        })
        .collect::<Vec<_>>();

    Ok(serde_json::to_string(&payload)?)
}

async fn add_task(pool: &SqlitePool, args: &serde_json::Value) -> ProviderResult<String> {
    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ProviderError::InvalidResponse("add_task: missing title".into()))?;
    let priority = args
        .get("priority")
        .and_then(|v| v.as_str())
        .unwrap_or("medium");
    let duration = args
        .get("duration_minutes")
        .and_then(|v| v.as_i64())
        .unwrap_or(15);
    let description = args.get("description").and_then(|v| v.as_str());

    let session_id = current_or_create_session(pool).await?;
    let next_order: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(order_index), -1) + 1 FROM tasks WHERE session_id = ?",
    )
    .bind(&session_id)
    .fetch_one(pool)
    .await?;

    let task_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO tasks (id, session_id, title, description, duration_minutes,
                            priority, order_index, status, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, 'pending', ?)",
    )
    .bind(&task_id)
    .bind(&session_id)
    .bind(title)
    .bind(description)
    .bind(duration)
    .bind(priority)
    .bind(next_order)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(serde_json::json!({
        "task_id": task_id,
        "title": title,
        "status": "pending"
    })
    .to_string())
}

async fn set_status(
    pool: &SqlitePool,
    args: &serde_json::Value,
    status: &str,
) -> ProviderResult<String> {
    let task_id = args
        .get("task_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ProviderError::InvalidResponse("missing task_id".into()))?;

    let now = Utc::now();
    let rows = match status {
        "in_progress" => sqlx::query(
            "UPDATE tasks SET status = 'in_progress',
                                  started_at = COALESCE(started_at, ?),
                                  completed_at = NULL
                 WHERE id = ?",
        )
        .bind(now)
        .bind(task_id)
        .execute(pool)
        .await?
        .rows_affected(),
        "completed" | "skipped" => {
            sqlx::query("UPDATE tasks SET status = ?, completed_at = ? WHERE id = ?")
                .bind(status)
                .bind(now)
                .bind(task_id)
                .execute(pool)
                .await?
                .rows_affected()
        }
        other => {
            return Err(ProviderError::InvalidResponse(format!(
                "set_status: unsupported status {other}"
            )))
        }
    };

    if rows == 0 {
        return Err(ProviderError::InvalidResponse(format!(
            "task {task_id} not found"
        )));
    }

    Ok(serde_json::json!({ "task_id": task_id, "status": status }).to_string())
}

async fn current_or_create_session(pool: &SqlitePool) -> ProviderResult<String> {
    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM sessions
         WHERE date(created_at) = date('now', 'localtime')
         ORDER BY created_at DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;

    if let Some((id,)) = existing {
        return Ok(id);
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO sessions (id, raw_input, language, created_at)
         VALUES (?, '(via chat)', 'auto', ?)",
    )
    .bind(&id)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(id)
}
