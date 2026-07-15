//! Google Tasks provider — the API-accessible successor to "Google Reminders"
//! (Reminders were folded into Tasks/Assistant; there is no standalone
//! Reminders API). Tasks map to Google Tasks with a native done state. Note
//! Google Tasks `due` is **date-only** — any time-of-day is dropped server-side.
//! Push only for now; `pull`/`full_pull` land with the Phase 2 read path.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::{json, Value};

use super::google_http::GoogleClient;
use crate::sync::error::{SyncError, SyncResult};
use crate::sync::keychain;
use crate::sync::provider::SyncProvider;
use crate::sync::types::{
    PullBatch, PushOutcome, PushResult, RemoteContainer, RemoteItem, SyncAccount, SyncOp,
    SyncProviderKind,
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
        _cursor: Option<String>,
        _linked_ids: Vec<String>,
    ) -> SyncResult<PullBatch> {
        Err(SyncError::NotImplemented(
            "google tasks pull is Phase 2".into(),
        ))
    }

    async fn full_pull(&self, _linked_ids: Vec<String>) -> SyncResult<PullBatch> {
        Err(SyncError::NotImplemented(
            "google tasks pull is Phase 2".into(),
        ))
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
