//! Google Calendar v3 provider. Tasks map to events; the Acorn task id and a
//! done flag round-trip through `extendedProperties.private` so a pulled event
//! can be matched back and its completion state recovered. Push only for now;
//! `pull`/`full_pull` land with the Phase 2 read path.

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde_json::{json, Value};

use super::google_http::GoogleClient;
use crate::sync::error::{SyncError, SyncResult};
use crate::sync::keychain;
use crate::sync::provider::SyncProvider;
use crate::sync::types::{
    PullBatch, PushOutcome, PushResult, RemoteContainer, RemoteItem, SyncAccount, SyncOp,
    SyncProviderKind,
};

const BASE: &str = "https://www.googleapis.com/calendar/v3";

pub struct GoogleCalendarProvider {
    client: GoogleClient,
    calendar_id: String,
}

impl GoogleCalendarProvider {
    pub async fn connect(account: &SyncAccount) -> SyncResult<Self> {
        let refresh = keychain::load_refresh_token(&account.provider, &account.id)?
            .ok_or_else(|| SyncError::MissingCredentials(account.account_label.clone()))?;
        let client = GoogleClient::connect(&refresh).await?;
        let calendar_id = account
            .container_id
            .clone()
            .unwrap_or_else(|| "primary".to_string());
        Ok(Self {
            client,
            calendar_id,
        })
    }

    fn events_url(&self) -> String {
        format!("{BASE}/calendars/{}/events", urlencode(&self.calendar_id))
    }

    fn event_url(&self, event_id: &str) -> String {
        format!(
            "{BASE}/calendars/{}/events/{}",
            urlencode(&self.calendar_id),
            urlencode(event_id)
        )
    }

    async fn create(&self, dedupe_key: &str, item: &RemoteItem) -> SyncResult<PushResult> {
        // Find-or-create: if a prior attempt already created the event (we
        // crashed before recording remote_id), adopt it instead of duplicating.
        if let Some(existing) = self.find_by_dedupe(dedupe_key).await? {
            return Ok(existing);
        }
        let body = event_body(item, Some(dedupe_key));
        let resp = self
            .client
            .send(self.client.post(&self.events_url()).json(&body))
            .await?;
        let value: Value = resp.json().await?;
        created_result(&value)
    }

    async fn update(
        &self,
        remote_id: &str,
        etag: Option<&str>,
        item: &RemoteItem,
    ) -> SyncResult<PushResult> {
        let body = event_body(item, None);
        let mut req = self.client.patch(&self.event_url(remote_id)).json(&body);
        if let Some(tag) = etag {
            req = req.header(reqwest::header::IF_MATCH, tag);
        }
        let resp = self.client.send(req).await?;
        let value: Value = resp.json().await?;
        Ok(PushResult::Updated {
            etag: string_field(&value, "etag"),
            updated: datetime_field(&value, "updated"),
        })
    }

    async fn delete(&self, remote_id: &str, etag: Option<&str>) -> SyncResult<PushResult> {
        let mut req = self.client.delete(&self.event_url(remote_id));
        if let Some(tag) = etag {
            req = req.header(reqwest::header::IF_MATCH, tag);
        }
        match self.client.send(req).await {
            Ok(_) => Ok(PushResult::Deleted),
            // The event is already gone — the delete goal is satisfied.
            Err(SyncError::Remote(msg)) if msg.starts_with("404") || msg.starts_with("410") => {
                Ok(PushResult::Deleted)
            }
            Err(SyncError::FullResyncRequired) => Ok(PushResult::Deleted),
            Err(SyncError::Conflict) => Ok(PushResult::Conflict),
            Err(other) => Err(other),
        }
    }

    async fn find_by_dedupe(&self, dedupe_key: &str) -> SyncResult<Option<PushResult>> {
        let resp = self
            .client
            .send(self.client.get(&self.events_url()).query(&[
                (
                    "privateExtendedProperty",
                    format!("acornDedupe={dedupe_key}").as_str(),
                ),
                ("showDeleted", "false"),
                ("maxResults", "1"),
            ]))
            .await?;
        let value: Value = resp.json().await?;
        let first = value
            .get("items")
            .and_then(Value::as_array)
            .and_then(|items| items.first());
        match first {
            Some(event) => Ok(Some(created_result(event)?)),
            None => Ok(None),
        }
    }
}

#[async_trait]
impl SyncProvider for GoogleCalendarProvider {
    fn kind(&self) -> SyncProviderKind {
        SyncProviderKind::GoogleCalendar
    }

    async fn list_containers(&self) -> SyncResult<Vec<RemoteContainer>> {
        let url = format!("{BASE}/users/me/calendarList");
        let resp = self.client.send(self.client.get(&url)).await?;
        let value: Value = resp.json().await?;
        let mut out = Vec::new();
        if let Some(items) = value.get("items").and_then(Value::as_array) {
            for item in items {
                let Some(id) = string_field(item, "id") else {
                    continue;
                };
                let name = string_field(item, "summary").unwrap_or_else(|| id.clone());
                let role = string_field(item, "accessRole").unwrap_or_default();
                out.push(RemoteContainer {
                    id,
                    name,
                    writable: matches!(role.as_str(), "owner" | "writer"),
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
                SyncOp::Create {
                    dedupe_key, item, ..
                } => self.create(dedupe_key, item).await,
                SyncOp::Update {
                    remote_id,
                    etag,
                    item,
                    ..
                } => self.update(remote_id, etag.as_deref(), item).await,
                SyncOp::Delete {
                    remote_id, etag, ..
                } => self.delete(remote_id, etag.as_deref()).await,
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
            "google calendar pull is Phase 2".into(),
        ))
    }

    async fn full_pull(&self, _linked_ids: Vec<String>) -> SyncResult<PullBatch> {
        Err(SyncError::NotImplemented(
            "google calendar pull is Phase 2".into(),
        ))
    }
}

fn event_body(item: &RemoteItem, dedupe_key: Option<&str>) -> Value {
    let mut private = serde_json::Map::new();
    private.insert("acornDone".into(), json!(item.completed.to_string()));
    if let Some(key) = dedupe_key {
        private.insert("acornDedupe".into(), json!(key));
    }
    let (start, end) = event_times(item);
    json!({
        "summary": item.title,
        "description": item.notes,
        "start": start,
        "end": end,
        "extendedProperties": { "private": private },
    })
}

/// Compute event start/end JSON. All-day events use exclusive end-of-next-day
/// per Google's convention; timed events default to a 30-minute block.
fn event_times(item: &RemoteItem) -> (Value, Value) {
    let start = item.start.or(item.due).unwrap_or_else(Utc::now);
    if item.all_day {
        let start_date = start.date_naive();
        let end_date = (start + Duration::days(1)).date_naive();
        (
            json!({ "date": start_date.to_string() }),
            json!({ "date": end_date.to_string() }),
        )
    } else {
        let end = item.end.unwrap_or(start + Duration::minutes(30));
        (
            json!({ "dateTime": start.to_rfc3339(), "timeZone": "UTC" }),
            json!({ "dateTime": end.to_rfc3339(), "timeZone": "UTC" }),
        )
    }
}

fn created_result(value: &Value) -> SyncResult<PushResult> {
    let remote_id = string_field(value, "id")
        .ok_or_else(|| SyncError::InvalidResponse("event response missing id".into()))?;
    Ok(PushResult::Created {
        remote_id,
        etag: string_field(value, "etag"),
        updated: datetime_field(value, "updated"),
    })
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

/// Minimal path-segment encoding for calendar/event ids (they can contain '@',
/// '#', etc. in `primary`/group calendars).
fn urlencode(segment: &str) -> String {
    let mut out = String::with_capacity(segment.len());
    for byte in segment.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}
