//! Google Tasks provider — the API-accessible successor to "Google Reminders"
//! (Reminders were folded into Tasks/Assistant; there is no standalone
//! Reminders API). Tasks map to Google Tasks with a native done state. Note
//! Google Tasks `due` is **date-only** — any time-of-day is dropped server-side.
//! The Tasks API has no syncToken, so pull uses an `updatedMin` high-water-mark
//! cursor (max `updated` seen, re-queried with a small overlap); 410 recovery
//! never fires here.

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde_json::{json, Value};

use super::google_http::GoogleClient;
use crate::sync::error::{SyncError, SyncResult};
use crate::sync::keychain;
use crate::sync::provider::SyncProvider;
use crate::sync::types::{
    PullBatch, PushOutcome, PushResult, RemoteChange, RemoteContainer, RemoteItem, SyncAccount,
    SyncOp, SyncProviderKind,
};

const BASE: &str = "https://tasks.googleapis.com/tasks/v1";

pub struct GoogleTasksProvider {
    client: GoogleClient,
    tasklist_id: String,
}

impl GoogleTasksProvider {
    pub async fn connect(account: &SyncAccount) -> SyncResult<Self> {
        let refresh = keychain::load_refresh_token(&account.provider, &account.id)?
            .ok_or_else(|| SyncError::MissingCredentials(account.account_label.clone()))?;
        let client = GoogleClient::connect(&refresh).await?;
        let tasklist_id = account
            .container_id
            .clone()
            .unwrap_or_else(|| "@default".to_string());
        Ok(Self {
            client,
            tasklist_id,
        })
    }

    fn tasks_url(&self) -> String {
        format!("{BASE}/lists/{}/tasks", self.tasklist_id)
    }

    fn task_url(&self, task_id: &str) -> String {
        format!("{BASE}/lists/{}/tasks/{task_id}", self.tasklist_id)
    }

    async fn create(&self, item: &RemoteItem) -> SyncResult<PushResult> {
        let resp = self
            .client
            .send(self.client.post(&self.tasks_url()).json(&task_body(item)))
            .await?;
        let value: Value = resp.json().await?;
        let remote_id = string_field(&value, "id")
            .ok_or_else(|| SyncError::InvalidResponse("task response missing id".into()))?;
        Ok(PushResult::Created {
            remote_id,
            etag: string_field(&value, "etag"),
            updated: datetime_field(&value, "updated"),
        })
    }

    async fn update(&self, remote_id: &str, item: &RemoteItem) -> SyncResult<PushResult> {
        let resp = self
            .client
            .send(
                self.client
                    .patch(&self.task_url(remote_id))
                    .json(&task_body(item)),
            )
            .await?;
        let value: Value = resp.json().await?;
        Ok(PushResult::Updated {
            etag: string_field(&value, "etag"),
            updated: datetime_field(&value, "updated"),
        })
    }

    async fn delete(&self, remote_id: &str) -> SyncResult<PushResult> {
        match self
            .client
            .send(self.client.delete(&self.task_url(remote_id)))
            .await
        {
            Ok(_) => Ok(PushResult::Deleted),
            Err(SyncError::Remote(msg)) if msg.starts_with("404") => Ok(PushResult::Deleted),
            Err(other) => Err(other),
        }
    }

    /// Walk `tasks.list`, optionally bounded below by an `updatedMin` derived
    /// from the cursor. The next cursor is the newest `updated` seen, clamped
    /// so it never regresses; with zero items the original cursor comes back
    /// unchanged.
    async fn list_tasks(&self, cursor: Option<String>) -> SyncResult<PullBatch> {
        let updated_min = cursor.as_deref().and_then(updated_min_from_cursor);
        let mut changes: Vec<RemoteChange> = Vec::new();
        let mut high_water: Option<DateTime<Utc>> = None;
        let mut page_token: Option<String> = None;
        loop {
            let mut query: Vec<(&str, String)> = vec![
                ("showDeleted", "true".to_string()),
                ("showHidden", "true".to_string()),
                ("showCompleted", "true".to_string()),
                ("maxResults", "100".to_string()),
            ];
            if let Some(min) = &updated_min {
                query.push(("updatedMin", min.clone()));
            }
            if let Some(page) = &page_token {
                query.push(("pageToken", page.clone()));
            }
            let resp = self
                .client
                .send(self.client.get(&self.tasks_url()).query(&query))
                .await?;
            let value: Value = resp.json().await?;
            if let Some(items) = value.get("items").and_then(Value::as_array) {
                for task in items {
                    let Some(change) = task_change(task) else {
                        continue;
                    };
                    if let Some(updated) = change.updated {
                        high_water = Some(high_water.map_or(updated, |mark| mark.max(updated)));
                    }
                    changes.push(change);
                }
            }
            page_token = string_field(&value, "nextPageToken");
            if page_token.is_none() {
                break;
            }
        }
        let next_cursor = next_cursor(cursor, high_water);
        Ok(PullBatch {
            changes,
            next_cursor,
        })
    }
}

#[async_trait]
impl SyncProvider for GoogleTasksProvider {
    fn kind(&self) -> SyncProviderKind {
        SyncProviderKind::GoogleTasks
    }

    async fn list_containers(&self) -> SyncResult<Vec<RemoteContainer>> {
        let url = format!("{BASE}/users/@me/lists");
        let resp = self.client.send(self.client.get(&url)).await?;
        let value: Value = resp.json().await?;
        let mut out = Vec::new();
        if let Some(items) = value.get("items").and_then(Value::as_array) {
            for item in items {
                let Some(id) = string_field(item, "id") else {
                    continue;
                };
                let name = string_field(item, "title").unwrap_or_else(|| id.clone());
                out.push(RemoteContainer {
                    id,
                    name,
                    writable: true,
                });
            }
        }
        Ok(out)
    }

    async fn push(&self, ops: Vec<SyncOp>) -> SyncResult<Vec<PushOutcome>> {
        let mut outcomes = Vec::with_capacity(ops.len());
        for op in ops {
            let link_id = op.link_id().to_string();
            let result = match &op {
                SyncOp::Create { item, .. } => self.create(item).await,
                SyncOp::Update {
                    remote_id, item, ..
                } => self.update(remote_id, item).await,
                SyncOp::Delete { remote_id, .. } => self.delete(remote_id).await,
            };
            outcomes.push(PushOutcome {
                link_id,
                result: result.unwrap_or_else(|e| PushResult::Failed {
                    message: e.to_string(),
                }),
            });
        }
        Ok(outcomes)
    }

    async fn pull(
        &self,
        cursor: Option<String>,
        _linked_ids: Vec<String>,
    ) -> SyncResult<PullBatch> {
        self.list_tasks(cursor).await
    }

    async fn full_pull(&self, _linked_ids: Vec<String>) -> SyncResult<PullBatch> {
        self.list_tasks(None).await
    }
}

fn task_body(item: &RemoteItem) -> Value {
    let status = if item.completed {
        "completed"
    } else {
        "needsAction"
    };
    let mut body = json!({
        "title": item.title,
        "notes": item.notes,
        "status": status,
    });
    // Google Tasks `due` is date-only; send midnight UTC of the due date.
    if let Some(due) = item.due {
        body["due"] = json!(format!("{}T00:00:00.000Z", due.date_naive()));
    }
    body
}

/// Derive the `updatedMin` bound from a cursor: parse the RFC3339 mark and back
/// it off by 5 seconds so same-second writes and minor clock skew are re-pulled
/// rather than skipped (re-applying a change is idempotent; missing one is
/// not). An unparseable cursor falls back to a full listing.
fn updated_min_from_cursor(cursor: &str) -> Option<String> {
    DateTime::parse_from_rfc3339(cursor)
        .ok()
        .map(|mark| (mark.with_timezone(&Utc) - Duration::seconds(5)).to_rfc3339())
}

/// Advance the high-water mark, never letting it regress: the 5-second overlap
/// re-fetches items just below the old mark, whose `updated` must not pull the
/// cursor backwards. With zero items the original cursor comes back unchanged.
fn next_cursor(cursor: Option<String>, high_water: Option<DateTime<Utc>>) -> Option<String> {
    let Some(new_mark) = high_water else {
        return cursor;
    };
    let old_mark = cursor
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));
    match old_mark {
        Some(old) if old > new_mark => cursor,
        _ => Some(new_mark.to_rfc3339()),
    }
}

/// Map one pulled task onto a [`RemoteChange`]. Tasks without an id (never seen
/// in practice) are skipped; `deleted: true` is the API's tombstone and carries
/// no payload.
fn task_change(task: &Value) -> Option<RemoteChange> {
    let remote_id = string_field(task, "id")?;
    let deleted = task
        .get("deleted")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let item = if deleted { None } else { Some(task_item(task)) };
    Some(RemoteChange {
        remote_id,
        etag: string_field(task, "etag"),
        updated: datetime_field(task, "updated"),
        deleted,
        item,
    })
}

fn task_item(task: &Value) -> RemoteItem {
    RemoteItem {
        title: string_field(task, "title").unwrap_or_default(),
        notes: string_field(task, "notes"),
        start: None,
        end: None,
        // Date-only semantics, but Google serializes it as a full RFC3339
        // timestamp (e.g. "2026-07-15T00:00:00.000Z").
        due: datetime_field(task, "due"),
        all_day: false,
        completed: task.get("status").and_then(Value::as_str) == Some("completed"),
        completed_at: datetime_field(task, "completed"),
    }
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

fn datetime_field(value: &Value, key: &str) -> Option<DateTime<Utc>> {
    value
        .get(key)
        .and_then(Value::as_str)
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
}
