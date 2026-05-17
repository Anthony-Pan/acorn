use chrono::Utc;
use tauri::State;
use uuid::Uuid;

use crate::db::models::{NewTaskInput, Subtask, Task, TaskStatus};
use crate::db::Database;
use crate::error::{AppError, AppResult};

#[tauri::command(rename_all = "camelCase")]
pub async fn insert_task(db: State<'_, Database>, input: NewTaskInput) -> AppResult<Task> {
    let task_id = Uuid::new_v4().to_string();
    let now = Utc::now();

    let mut tx = db.pool().begin().await?;

    sqlx::query(
        "INSERT INTO tasks (
            id, session_id, title, description, duration_minutes,
            priority, order_index, status, created_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, 'pending', ?)",
    )
    .bind(&task_id)
    .bind(&input.session_id)
    .bind(&input.title)
    .bind(&input.description)
    .bind(input.duration_minutes)
    .bind(input.priority)
    .bind(input.order_index)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    for (idx, subtask) in input.subtasks.iter().enumerate() {
        sqlx::query(
            "INSERT INTO subtasks (id, task_id, title, done, order_index) VALUES (?, ?, ?, 0, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&task_id)
        .bind(&subtask.title)
        .bind(idx as i64)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?")
        .bind(&task_id)
        .fetch_one(db.pool())
        .await
        .map_err(Into::into)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_task_status(
    db: State<'_, Database>,
    task_id: String,
    status: TaskStatus,
) -> AppResult<Task> {
    let now = Utc::now();

    let rows = match status {
        TaskStatus::InProgress => {
            sqlx::query(
                "UPDATE tasks SET status = ?, started_at = COALESCE(started_at, ?), completed_at = NULL WHERE id = ?",
            )
            .bind(status)
            .bind(now)
            .bind(&task_id)
            .execute(db.pool())
            .await?
            .rows_affected()
        }
        TaskStatus::Completed | TaskStatus::Skipped => {
            sqlx::query("UPDATE tasks SET status = ?, completed_at = ? WHERE id = ?")
                .bind(status)
                .bind(now)
                .bind(&task_id)
                .execute(db.pool())
                .await?
                .rows_affected()
        }
        TaskStatus::Pending => {
            sqlx::query(
                "UPDATE tasks SET status = ?, started_at = NULL, completed_at = NULL WHERE id = ?",
            )
            .bind(status)
            .bind(&task_id)
            .execute(db.pool())
            .await?
            .rows_affected()
        }
    };

    if rows == 0 {
        return Err(AppError::NotFound(format!("task {task_id}")));
    }

    sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?")
        .bind(&task_id)
        .fetch_one(db.pool())
        .await
        .map_err(Into::into)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn list_subtasks(
    db: State<'_, Database>,
    task_id: String,
) -> AppResult<Vec<Subtask>> {
    sqlx::query_as::<_, Subtask>(
        "SELECT * FROM subtasks WHERE task_id = ? ORDER BY order_index ASC",
    )
    .bind(&task_id)
    .fetch_all(db.pool())
    .await
    .map_err(Into::into)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn toggle_subtask(
    db: State<'_, Database>,
    subtask_id: String,
) -> AppResult<Subtask> {
    let rows = sqlx::query("UPDATE subtasks SET done = 1 - done WHERE id = ?")
        .bind(&subtask_id)
        .execute(db.pool())
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound(format!("subtask {subtask_id}")));
    }

    sqlx::query_as::<_, Subtask>("SELECT * FROM subtasks WHERE id = ?")
        .bind(&subtask_id)
        .fetch_one(db.pool())
        .await
        .map_err(Into::into)
}
