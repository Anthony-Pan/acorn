use chrono::{DateTime, Utc};
use sqlx::Sqlite;
use tauri::State;
use uuid::Uuid;

use crate::db::models::{NewTaskInput, Subtask, Task, TaskStatus};
use crate::db::Database;
use crate::error::{AppError, AppResult};

/// The write choke point: mark every live sync link for a task dirty so the sync
/// engine re-pushes it. Callers pair this with an `updated_at` bump on the task
/// itself inside the same transaction, so a task is never silently left clean
/// toward a provider it has drifted from. A no-op (0 rows) when no accounts are
/// connected, so it stays cheap on the common path.
async fn dirty_task_links<'e, E>(executor: E, task_id: &str) -> Result<(), sqlx::Error>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    sqlx::query(
        "UPDATE sync_links
            SET local_rev = local_rev + 1,
                sync_state = CASE WHEN remote_id IS NULL THEN 'pending_create' ELSE 'pending_update' END
          WHERE task_id = ? AND deleted_at IS NULL",
    )
    .bind(task_id)
    .execute(executor)
    .await?;
    Ok(())
}

async fn fetch_task(db: &Database, task_id: &str) -> AppResult<Task> {
    sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?")
        .bind(task_id)
        .fetch_one(db.pool())
        .await
        .map_err(Into::into)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn insert_task(db: State<'_, Database>, input: NewTaskInput) -> AppResult<Task> {
    let task_id = Uuid::new_v4().to_string();
    let now = Utc::now();

    let mut tx = db.pool().begin().await?;

    sqlx::query(
        "INSERT INTO tasks (
            id, session_id, title, description, duration_minutes,
            priority, order_index, status, created_at, updated_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, 'pending', ?, ?)",
    )
    .bind(&task_id)
    .bind(&input.session_id)
    .bind(&input.title)
    .bind(&input.description)
    .bind(input.duration_minutes)
    .bind(input.priority)
    .bind(input.order_index)
    .bind(now)
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

    // No links exist for a brand-new task; the engine's reconcile pass creates
    // pending_create links for each enabled account on the next cycle.
    fetch_task(&db, &task_id).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_task_status(
    db: State<'_, Database>,
    task_id: String,
    status: TaskStatus,
) -> AppResult<Task> {
    let now = Utc::now();
    let mut tx = db.pool().begin().await?;

    let rows = match status {
        TaskStatus::InProgress => {
            sqlx::query(
                "UPDATE tasks SET status = ?, started_at = COALESCE(started_at, ?), completed_at = NULL, updated_at = ? WHERE id = ?",
            )
            .bind(status)
            .bind(now)
            .bind(now)
            .bind(&task_id)
            .execute(&mut *tx)
            .await?
            .rows_affected()
        }
        TaskStatus::Completed | TaskStatus::Skipped => {
            sqlx::query("UPDATE tasks SET status = ?, completed_at = ?, updated_at = ? WHERE id = ?")
                .bind(status)
                .bind(now)
                .bind(now)
                .bind(&task_id)
                .execute(&mut *tx)
                .await?
                .rows_affected()
        }
        TaskStatus::Pending => {
            sqlx::query(
                "UPDATE tasks SET status = ?, started_at = NULL, completed_at = NULL, updated_at = ? WHERE id = ?",
            )
            .bind(status)
            .bind(now)
            .bind(&task_id)
            .execute(&mut *tx)
            .await?
            .rows_affected()
        }
    };

    if rows == 0 {
        return Err(AppError::NotFound(format!("task {task_id}")));
    }

    dirty_task_links(&mut *tx, &task_id).await?;
    tx.commit().await?;

    fetch_task(&db, &task_id).await
}

/// Set (or clear) a task's schedule so it can be synced as a calendar event
/// (`scheduled_start`/`scheduled_end`) or a reminder/task (`due_date`).
#[tauri::command(rename_all = "camelCase")]
pub async fn set_task_schedule(
    db: State<'_, Database>,
    task_id: String,
    scheduled_start: Option<DateTime<Utc>>,
    scheduled_end: Option<DateTime<Utc>>,
    due_date: Option<DateTime<Utc>>,
    all_day: bool,
) -> AppResult<Task> {
    let now = Utc::now();
    let mut tx = db.pool().begin().await?;

    let rows = sqlx::query(
        "UPDATE tasks
            SET scheduled_start = ?, scheduled_end = ?, due_date = ?, all_day = ?, updated_at = ?
          WHERE id = ?",
    )
    .bind(scheduled_start)
    .bind(scheduled_end)
    .bind(due_date)
    .bind(all_day)
    .bind(now)
    .bind(&task_id)
    .execute(&mut *tx)
    .await?
    .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound(format!("task {task_id}")));
    }

    dirty_task_links(&mut *tx, &task_id).await?;
    tx.commit().await?;

    fetch_task(&db, &task_id).await
}

/// Delete a task and propagate the delete to every provider it was synced to.
///
/// Links that never reached a remote are dropped outright; links with a
/// `remote_id` are tombstoned (`pending_delete`) so the engine can push the
/// remote delete. The task row is then removed — subtasks cascade, and the
/// `ON DELETE SET NULL` on `sync_links.task_id` keeps the tombstones alive with
/// their `remote_id` so the delete still ships after the domain row is gone.
#[tauri::command(rename_all = "camelCase")]
pub async fn delete_task(db: State<'_, Database>, task_id: String) -> AppResult<()> {
    let now = Utc::now();
    let mut tx = db.pool().begin().await?;

    sqlx::query("DELETE FROM sync_links WHERE task_id = ? AND remote_id IS NULL")
        .bind(&task_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "UPDATE sync_links
            SET sync_state = 'pending_delete', deleted_origin = 'local',
                deleted_at = ?, local_rev = local_rev + 1
          WHERE task_id = ? AND deleted_at IS NULL",
    )
    .bind(now)
    .bind(&task_id)
    .execute(&mut *tx)
    .await?;

    let rows = sqlx::query("DELETE FROM tasks WHERE id = ?")
        .bind(&task_id)
        .execute(&mut *tx)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound(format!("task {task_id}")));
    }

    tx.commit().await?;
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn list_subtasks(db: State<'_, Database>, task_id: String) -> AppResult<Vec<Subtask>> {
    sqlx::query_as::<_, Subtask>(
        "SELECT * FROM subtasks WHERE task_id = ? ORDER BY order_index ASC",
    )
    .bind(&task_id)
    .fetch_all(db.pool())
    .await
    .map_err(Into::into)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn toggle_subtask(db: State<'_, Database>, subtask_id: String) -> AppResult<Subtask> {
    let now = Utc::now();
    let mut tx = db.pool().begin().await?;

    let rows = sqlx::query("UPDATE subtasks SET done = 1 - done WHERE id = ?")
        .bind(&subtask_id)
        .execute(&mut *tx)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound(format!("subtask {subtask_id}")));
    }

    // A subtask toggle changes the parent task's synced content, so touch the
    // parent's clock and dirty its links.
    if let Some((task_id,)) =
        sqlx::query_as::<_, (String,)>("SELECT task_id FROM subtasks WHERE id = ?")
            .bind(&subtask_id)
            .fetch_optional(&mut *tx)
            .await?
    {
        sqlx::query("UPDATE tasks SET updated_at = ? WHERE id = ?")
            .bind(now)
            .bind(&task_id)
            .execute(&mut *tx)
            .await?;
        dirty_task_links(&mut *tx, &task_id).await?;
    }

    tx.commit().await?;

    sqlx::query_as::<_, Subtask>("SELECT * FROM subtasks WHERE id = ?")
        .bind(&subtask_id)
        .fetch_one(db.pool())
        .await
        .map_err(Into::into)
}
