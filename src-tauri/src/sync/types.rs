use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::error::SyncError;

/// Which external system a connected account talks to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncProviderKind {
    AppleCalendar,
    AppleReminders,
    GoogleCalendar,
    GoogleTasks,
}

impl SyncProviderKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "apple_calendar" => Some(Self::AppleCalendar),
            "apple_reminders" => Some(Self::AppleReminders),
            "google_calendar" => Some(Self::GoogleCalendar),
            "google_tasks" => Some(Self::GoogleTasks),
            _ => None,
        }
    }

    pub fn is_google(self) -> bool {
        matches!(self, Self::GoogleCalendar | Self::GoogleTasks)
    }

    pub fn is_apple(self) -> bool {
        matches!(self, Self::AppleCalendar | Self::AppleReminders)
    }

    /// The target kind this provider is inherently about. Calendars map tasks to
    /// timed events; reminders/tasks map them to due-dated checklist items.
    pub fn natural_target(self) -> TargetKind {
        match self {
            Self::AppleCalendar | Self::GoogleCalendar => TargetKind::Event,
            Self::AppleReminders | Self::GoogleTasks => TargetKind::Reminder,
        }
    }
}

/// How a task is projected onto the remote.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetKind {
    Event,
    Reminder,
}

impl TargetKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Event => "event",
            Self::Reminder => "reminder",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "event" => Some(Self::Event),
            "reminder" => Some(Self::Reminder),
            _ => None,
        }
    }
}

/// A selectable remote container: a Google calendar, a Google task list, or an
/// `EKCalendar` (calendar or reminders list).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteContainer {
    pub id: String,
    pub name: String,
    pub writable: bool,
}

/// Provider-agnostic projection of a task onto a remote item. Used both to push
/// (local → remote) and to represent a pulled remote (remote → local).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RemoteItem {
    pub title: String,
    pub notes: Option<String>,
    /// Event mapping: start of the timed block.
    pub start: Option<DateTime<Utc>>,
    /// Event mapping: end of the block.
    pub end: Option<DateTime<Utc>>,
    /// Reminder/task mapping: due date (Google Tasks honours the date only).
    pub due: Option<DateTime<Utc>>,
    pub all_day: bool,
    pub completed: bool,
    pub completed_at: Option<DateTime<Utc>>,
}

/// One push operation handed to a provider.
#[derive(Debug, Clone)]
pub enum SyncOp {
    Create {
        link_id: String,
        dedupe_key: String,
        item: RemoteItem,
    },
    Update {
        link_id: String,
        remote_id: String,
        etag: Option<String>,
        item: RemoteItem,
    },
    Delete {
        link_id: String,
        remote_id: String,
        etag: Option<String>,
    },
}

impl SyncOp {
    pub fn link_id(&self) -> &str {
        match self {
            Self::Create { link_id, .. }
            | Self::Update { link_id, .. }
            | Self::Delete { link_id, .. } => link_id,
        }
    }
}

/// Outcome of a single push op, keyed back to the link it came from.
#[derive(Debug, Clone)]
pub struct PushOutcome {
    pub link_id: String,
    pub result: PushResult,
}

#[derive(Debug, Clone)]
pub enum PushResult {
    Created {
        remote_id: String,
        etag: Option<String>,
        updated: Option<DateTime<Utc>>,
    },
    Updated {
        etag: Option<String>,
        updated: Option<DateTime<Utc>>,
    },
    Deleted,
    /// Remote rejected the write on a version check (etag mismatch / 412).
    Conflict,
    /// Non-fatal per-item failure; the engine records it and retries later.
    Failed {
        message: String,
    },
}

/// A batch of remote changes from an incremental (or full) pull.
#[derive(Debug, Clone, Default)]
pub struct PullBatch {
    pub changes: Vec<RemoteChange>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RemoteChange {
    pub remote_id: String,
    pub etag: Option<String>,
    pub updated: Option<DateTime<Utc>>,
    pub deleted: bool,
    pub item: Option<RemoteItem>,
}

/// Streamed progress for `trigger_sync_now`, bridged to a `Channel<SyncEvent>`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum SyncEvent {
    Started {
        account_id: String,
        account_label: String,
    },
    Pushed {
        account_id: String,
        created: u32,
        updated: u32,
        deleted: u32,
        failed: u32,
    },
    Pulled {
        account_id: String,
        applied: u32,
        deleted: u32,
        conflicts: u32,
    },
    Finished {
        account_id: String,
    },
    Error {
        account_id: String,
        message: String,
    },
}

/// A connected account row (`sync_accounts`). Serialized to the frontend
/// presence-only — it never carries token bytes.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SyncAccount {
    pub id: String,
    pub provider: String,
    pub target_kind: String,
    pub account_label: String,
    pub container_id: Option<String>,
    pub container_name: Option<String>,
    pub enabled: bool,
    pub has_token: bool,
    pub conflict_policy: String,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl SyncAccount {
    pub fn kind(&self) -> Option<SyncProviderKind> {
        SyncProviderKind::parse(&self.provider)
    }

    pub fn target(&self) -> TargetKind {
        TargetKind::parse(&self.target_kind).unwrap_or(TargetKind::Event)
    }
}

/// A per-task ↔ per-account link row (`sync_links`). A full mirror of the table;
/// some fields are only consumed by the Phase 2 pull / conflict path.
#[allow(dead_code)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SyncLink {
    pub id: String,
    pub task_id: Option<String>,
    pub account_id: String,
    pub remote_id: Option<String>,
    pub remote_etag: Option<String>,
    pub remote_updated: Option<DateTime<Utc>>,
    pub content_hash: Option<String>,
    pub dedupe_key: Option<String>,
    pub local_rev: i64,
    pub synced_local_rev: i64,
    pub sync_state: String,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub retry_count: i64,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_origin: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Status snapshot returned by `get_sync_status`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    pub account_id: String,
    pub enabled: bool,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub pending: i64,
    /// Links parked in `sync_state = 'conflict'` awaiting a user decision.
    pub conflicts: i64,
}

/// Guard against a provider string the catalogue doesn't know.
pub fn require_kind(provider: &str) -> Result<SyncProviderKind, SyncError> {
    SyncProviderKind::parse(provider)
        .ok_or_else(|| SyncError::Unavailable(format!("unknown provider '{provider}'")))
}
