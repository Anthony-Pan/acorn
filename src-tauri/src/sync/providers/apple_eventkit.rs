//! Apple EventKit provider (macOS) — calendars and reminders via `EKEventStore`.
//!
//! # Threading / `Send + Sync`
//!
//! The [`SyncProvider`] trait requires `Send + Sync`, but every objc2 handle
//! (`Retained<EKEventStore>`, `Retained<EKCalendar>`, …) is `!Send`. The
//! provider therefore holds **no Objective-C state at all** — only plain Rust
//! data (`kind`, `container_id`). Each operation spawns onto
//! `tokio::task::spawn_blocking`, creates a fresh `EKEventStore` on that
//! blocking thread, and lets every objc2 value live and die there. This keeps
//! the async futures `Send` and never parks the main thread on EventKit.
//!
//! # Authorization
//!
//! Uses the macOS 14+ full-access APIs (`requestFullAccessToEvents…` /
//! `requestFullAccessToReminders…`, available in objc2-event-kit 0.3). The
//! completion block is bridged back to the blocking thread with a
//! `std::sync::mpsc` channel and a 60 s `recv_timeout`. Denied / restricted /
//! write-only states map to [`SyncError::Unauthorized`] with a message that
//! points the user at System Settings > Privacy & Security.
//!
//! # Remote identifier choice
//!
//! `remote_id` is the item's `calendarItemIdentifier`: a locally-unique handle
//! that round-trips through `calendarItemWithIdentifier:`, which is what both
//! the push path (create / update / delete) and the pull path key on. It is
//! not sync-proof across devices or full resyncs — a later phase can
//! additionally store `calendarItemExternalIdentifier` for cross-device
//! matching.
//!
//! # Pull — per-link verify
//!
//! The classic EventKit API has no change feed or sync token, so `pull` /
//! `full_pull` re-verify every linked `remote_id` instead of walking a delta:
//! each id is looked up with `calendarItemWithIdentifier:` — a miss becomes a
//! deletion tombstone, a hit becomes a full [`RemoteItem`] snapshot with
//! `lastModifiedDate` as `updated`, which feeds the engine's last-writer-wins
//! conflict resolution. Unchanged items are returned too; the engine's
//! content-hash echo suppression drops the no-ops. There is no cursor —
//! `next_cursor` is always `None`, and `full_pull` is the same verify pass.

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use block2::RcBlock;
use chrono::{DateTime, TimeZone, Utc};
use objc2::rc::Retained;
use objc2::runtime::Bool;
use objc2_event_kit::{
    EKAuthorizationStatus, EKCalendar, EKCalendarItem, EKEntityType, EKEvent, EKEventStore,
    EKReminder, EKSpan,
};
use objc2_foundation::{NSCalendar, NSCalendarUnit, NSDate, NSDateComponents, NSError, NSString};

use crate::sync::error::{SyncError, SyncResult};
use crate::sync::provider::SyncProvider;
use crate::sync::types::{
    PullBatch, PushOutcome, PushResult, RemoteChange, RemoteContainer, RemoteItem, SyncAccount,
    SyncOp, SyncProviderKind,
};

/// How long we wait for the user to answer the one-time permission prompt.
const AUTH_PROMPT_TIMEOUT: Duration = Duration::from_secs(60);

/// Default block length when a task has a start but no end.
const DEFAULT_EVENT_DURATION_SECS: f64 = 30.0 * 60.0;

/// All-day window: local start-of-day .. one second before the next midnight,
/// so the event stays inside a single calendar day.
const ALL_DAY_WINDOW_SECS: f64 = 86_399.0;

/// Apple EventKit sync provider. Holds only `Send + Sync` Rust data — see the
/// module docs for why no `EKEventStore` lives in the struct.
pub struct AppleEventKitProvider {
    kind: SyncProviderKind,
    /// `EKCalendar.calendarIdentifier` chosen by the user; `None` (or a stale
    /// identifier) falls back to the system default calendar / reminders list.
    container_id: Option<String>,
}

impl AppleEventKitProvider {
    pub fn new(account: &SyncAccount) -> SyncResult<Self> {
        let kind = account
            .kind()
            .filter(|k| k.is_apple())
            .ok_or_else(|| SyncError::Unavailable("not an Apple provider".into()))?;
        Ok(Self {
            kind,
            container_id: account.container_id.clone(),
        })
    }
}

#[async_trait]
impl SyncProvider for AppleEventKitProvider {
    fn kind(&self) -> SyncProviderKind {
        self.kind
    }

    async fn list_containers(&self) -> SyncResult<Vec<RemoteContainer>> {
        let entity = entity_type(self.kind);
        run_blocking(move || list_containers_blocking(entity)).await
    }

    async fn push(&self, ops: Vec<SyncOp>) -> SyncResult<Vec<PushOutcome>> {
        let kind = self.kind;
        let container_id = self.container_id.clone();
        run_blocking(move || push_blocking(kind, container_id, ops)).await
    }

    /// EventKit has no change feed, so the cursor is meaningless here — every
    /// pull is a per-link verify pass and `next_cursor` stays `None`.
    async fn pull(
        &self,
        _cursor: Option<String>,
        linked_ids: Vec<String>,
    ) -> SyncResult<PullBatch> {
        if linked_ids.is_empty() {
            // Nothing is linked yet — do not touch EventKit (no store, no
            // permission prompt) just to return an empty batch.
            return Ok(PullBatch::default());
        }
        let kind = self.kind;
        run_blocking(move || pull_blocking(kind, linked_ids)).await
    }

    async fn full_pull(&self, linked_ids: Vec<String>) -> SyncResult<PullBatch> {
        // A "full" pull is the same verify pass — there is no cursor to reset.
        self.pull(None, linked_ids).await
    }
}

/// Run an EventKit closure on the blocking pool. All objc2 values must be
/// created *and dropped* inside `op`; only plain Rust data crosses the join.
async fn run_blocking<T, F>(op: F) -> SyncResult<T>
where
    F: FnOnce() -> SyncResult<T> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(op)
        .await
        .map_err(|e| SyncError::Remote(format!("EventKit blocking task failed to join: {e}")))?
}

/// The EKEntityType for an account kind. The constructor guarantees `kind`
/// is one of the two Apple variants.
fn entity_type(kind: SyncProviderKind) -> EKEntityType {
    if kind == SyncProviderKind::AppleReminders {
        EKEntityType::Reminder
    } else {
        EKEntityType::Event
    }
}

/// Human label matching the System Settings > Privacy & Security pane names.
fn entity_label(entity: EKEntityType) -> &'static str {
    if entity == EKEntityType::Reminder {
        "Reminders"
    } else {
        "Calendars"
    }
}

fn unauthorized(entity: EKEntityType) -> SyncError {
    let label = entity_label(entity);
    SyncError::Unauthorized(format!(
        "Acorn does not have {label} access. Enable it in \
         System Settings > Privacy & Security > {label}, then sync again."
    ))
}

fn new_store() -> Retained<EKEventStore> {
    // SAFETY: Plain Objective-C constructor with no arguments; EKEventStore
    // may be created on any thread.
    unsafe { EKEventStore::new() }
}

// ---------------------------------------------------------------------------
// Authorization
// ---------------------------------------------------------------------------

/// Check (and if undetermined, request) full access for `entity`. Must run on
/// the blocking pool: it may park up to [`AUTH_PROMPT_TIMEOUT`] waiting for
/// the user's answer to the system prompt.
fn ensure_authorized_blocking(store: &EKEventStore, entity: EKEntityType) -> SyncResult<()> {
    // SAFETY: Class method returning a plain enum value; no object state.
    let status = unsafe { EKEventStore::authorizationStatusForEntityType(entity) };
    if status == EKAuthorizationStatus::FullAccess {
        return Ok(());
    }
    if status == EKAuthorizationStatus::NotDetermined {
        return request_full_access_blocking(store, entity);
    }
    // Denied, Restricted, or WriteOnly — write-only cannot read items back by
    // identifier, so it is not enough for sync either.
    Err(unauthorized(entity))
}

/// Show the one-time system permission prompt and wait for the outcome. Uses
/// the macOS 14+ `requestFullAccessTo…` APIs, which objc2-event-kit 0.3
/// exposes directly (the deprecated `requestAccessToEntityType:completion:`
/// is not needed).
fn request_full_access_blocking(store: &EKEventStore, entity: EKEntityType) -> SyncResult<()> {
    let (tx, rx) = mpsc::channel::<Result<bool, String>>();
    // The completion is Fn (in principle re-invokable), so the sender is taken
    // out of the slot exactly once.
    let tx_slot = Arc::new(Mutex::new(Some(tx)));

    let block = {
        let slot = Arc::clone(&tx_slot);
        RcBlock::new(move |granted: Bool, error: *mut NSError| {
            let payload = if error.is_null() {
                Ok(granted.as_bool())
            } else {
                // SAFETY: EventKit passes a valid NSError when non-null; it is
                // only read for the duration of this callback.
                Err(unsafe { (*error).localizedDescription().to_string() })
            };
            if let Ok(mut guard) = slot.lock() {
                if let Some(sender) = guard.take() {
                    let _ = sender.send(payload);
                }
            }
        })
    };

    // SAFETY: `RcBlock::as_ptr` yields a valid block pointer; EventKit copies
    // the block and invokes it once on an internal queue. We are on a tokio
    // blocking thread (never the main thread), so parking on the channel below
    // cannot deadlock the callback delivery.
    unsafe {
        if entity == EKEntityType::Reminder {
            store.requestFullAccessToRemindersWithCompletion(RcBlock::as_ptr(&block));
        } else {
            store.requestFullAccessToEventsWithCompletion(RcBlock::as_ptr(&block));
        }
    }

    match rx.recv_timeout(AUTH_PROMPT_TIMEOUT) {
        Ok(Ok(true)) => Ok(()),
        Ok(Ok(false)) => Err(unauthorized(entity)),
        Ok(Err(message)) => Err(SyncError::Unauthorized(format!(
            "{} access request failed: {message}",
            entity_label(entity)
        ))),
        Err(_) => Err(SyncError::Unauthorized(format!(
            "Timed out waiting for the {} permission prompt. Enable access in \
             System Settings > Privacy & Security, then sync again.",
            entity_label(entity)
        ))),
    }
}

// ---------------------------------------------------------------------------
// Containers
// ---------------------------------------------------------------------------

fn list_containers_blocking(entity: EKEntityType) -> SyncResult<Vec<RemoteContainer>> {
    let store = new_store();
    ensure_authorized_blocking(&store, entity)?;

    // SAFETY: Query on a valid store; returns a retained array of calendars.
    let calendars = unsafe { store.calendarsForEntityType(entity) };
    let containers = calendars
        .iter()
        .map(|calendar| {
            // SAFETY: Plain property getters on a valid EKCalendar.
            unsafe {
                RemoteContainer {
                    id: calendar.calendarIdentifier().to_string(),
                    name: calendar.title().to_string(),
                    writable: calendar.allowsContentModifications(),
                }
            }
        })
        .collect();
    Ok(containers)
}

/// Pick the calendar to write into: the configured `container_id` when it
/// still resolves, otherwise the system default for the entity type (also the
/// fallback for a stale identifier, e.g. a list the user deleted).
fn resolve_calendar(
    store: &EKEventStore,
    entity: EKEntityType,
    container_id: Option<&str>,
) -> SyncResult<Retained<EKCalendar>> {
    if let Some(id) = container_id {
        let ns_id = NSString::from_str(id);
        // SAFETY: Identifier lookup on a valid store; returns nil when unknown.
        if let Some(calendar) = unsafe { store.calendarWithIdentifier(&ns_id) } {
            return Ok(calendar);
        }
    }
    // SAFETY: Default-calendar getters on a valid store; nil when the user has
    // no default (e.g. no account configured for this entity type).
    let fallback = unsafe {
        if entity == EKEntityType::Reminder {
            store.defaultCalendarForNewReminders()
        } else {
            store.defaultCalendarForNewEvents()
        }
    };
    fallback.ok_or_else(|| {
        SyncError::Unavailable(format!(
            "no default {} list is available to sync into",
            entity_label(entity)
        ))
    })
}

// ---------------------------------------------------------------------------
// Push
// ---------------------------------------------------------------------------

fn push_blocking(
    kind: SyncProviderKind,
    container_id: Option<String>,
    ops: Vec<SyncOp>,
) -> SyncResult<Vec<PushOutcome>> {
    let store = new_store();
    let entity = entity_type(kind);
    ensure_authorized_blocking(&store, entity)?;
    let calendar = resolve_calendar(&store, entity, container_id.as_deref())?;

    let outcomes = ops
        .into_iter()
        .map(|op| {
            let link_id = op.link_id().to_string();
            // Per-item errors become PushResult::Failed instead of aborting
            // the batch; the engine records them and retries later.
            let result = apply_op(&store, kind, &calendar, op)
                .unwrap_or_else(|message| PushResult::Failed { message });
            PushOutcome { link_id, result }
        })
        .collect();
    Ok(outcomes)
}

fn apply_op(
    store: &EKEventStore,
    kind: SyncProviderKind,
    calendar: &EKCalendar,
    op: SyncOp,
) -> Result<PushResult, String> {
    let reminders = kind == SyncProviderKind::AppleReminders;
    match op {
        SyncOp::Create { item, .. } => {
            if reminders {
                create_reminder(store, calendar, &item)
            } else {
                create_event(store, calendar, &item)
            }
        }
        SyncOp::Update {
            remote_id, item, ..
        } => update_item(store, reminders, &remote_id, &item),
        SyncOp::Delete { remote_id, .. } => delete_item(store, reminders, &remote_id),
    }
}

fn create_event(
    store: &EKEventStore,
    calendar: &EKCalendar,
    item: &RemoteItem,
) -> Result<PushResult, String> {
    // SAFETY: Factory method on a valid store; the returned event is bound to
    // this store, as saveEvent requires.
    let event = unsafe { EKEvent::eventWithEventStore(store) };
    // SAFETY: Assigning a calendar fetched from the same store.
    unsafe { event.setCalendar(Some(calendar)) };
    apply_event_fields(&event, item)?;
    save_event(store, &event)?;

    // SAFETY: Property getter on the just-saved event; always non-nil after a
    // successful save.
    let remote_id = unsafe { event.calendarItemIdentifier() }.to_string();
    Ok(PushResult::Created {
        remote_id,
        etag: None,
        updated: last_modified(&event),
    })
}

fn create_reminder(
    store: &EKEventStore,
    calendar: &EKCalendar,
    item: &RemoteItem,
) -> Result<PushResult, String> {
    // SAFETY: Factory method on a valid store; the returned reminder is bound
    // to this store, as saveReminder requires.
    let reminder = unsafe { EKReminder::reminderWithEventStore(store) };
    // SAFETY: Assigning a calendar fetched from the same store.
    unsafe { reminder.setCalendar(Some(calendar)) };
    apply_reminder_fields(&reminder, item);
    save_reminder(store, &reminder)?;

    // SAFETY: Property getter on the just-saved reminder.
    let remote_id = unsafe { reminder.calendarItemIdentifier() }.to_string();
    Ok(PushResult::Created {
        remote_id,
        etag: None,
        updated: last_modified(&reminder),
    })
}

fn update_item(
    store: &EKEventStore,
    reminders: bool,
    remote_id: &str,
    item: &RemoteItem,
) -> Result<PushResult, String> {
    let existing = fetch_item(store, remote_id).ok_or("item vanished")?;
    if reminders {
        let reminder = existing
            .downcast::<EKReminder>()
            .map_err(|_| "remote item is not a reminder".to_string())?;
        apply_reminder_fields(&reminder, item);
        save_reminder(store, &reminder)?;
        Ok(PushResult::Updated {
            etag: None,
            updated: last_modified(&reminder),
        })
    } else {
        let event = existing
            .downcast::<EKEvent>()
            .map_err(|_| "remote item is not an event".to_string())?;
        apply_event_fields(&event, item)?;
        save_event(store, &event)?;
        Ok(PushResult::Updated {
            etag: None,
            updated: last_modified(&event),
        })
    }
}

fn delete_item(
    store: &EKEventStore,
    reminders: bool,
    remote_id: &str,
) -> Result<PushResult, String> {
    let Some(existing) = fetch_item(store, remote_id) else {
        // Already gone — the delete's goal is satisfied.
        return Ok(PushResult::Deleted);
    };
    if reminders {
        let reminder = existing
            .downcast::<EKReminder>()
            .map_err(|_| "remote item is not a reminder".to_string())?;
        // SAFETY: Removing a reminder fetched from this store, committing
        // immediately.
        unsafe { store.removeReminder_commit_error(&reminder, true) }
            .map_err(|e| e.localizedDescription().to_string())?;
    } else {
        let event = existing
            .downcast::<EKEvent>()
            .map_err(|_| "remote item is not an event".to_string())?;
        // SAFETY: Removing an event fetched from this store; only this
        // occurrence, committing immediately.
        unsafe { store.removeEvent_span_commit_error(&event, EKSpan::ThisEvent, true) }
            .map_err(|e| e.localizedDescription().to_string())?;
    }
    Ok(PushResult::Deleted)
}

fn fetch_item(store: &EKEventStore, remote_id: &str) -> Option<Retained<EKCalendarItem>> {
    let ns_id = NSString::from_str(remote_id);
    // SAFETY: Identifier lookup on a valid store; returns nil when the item no
    // longer exists.
    unsafe { store.calendarItemWithIdentifier(&ns_id) }
}

// ---------------------------------------------------------------------------
// Pull — per-link verify
// ---------------------------------------------------------------------------

/// Verify every linked remote id against ONE store in a single blocking pass.
/// A missing id is an external deletion; a resolved one becomes a full
/// snapshot. All ids come back as changes — the engine's content-hash echo
/// suppression skips the unchanged ones.
fn pull_blocking(kind: SyncProviderKind, linked_ids: Vec<String>) -> SyncResult<PullBatch> {
    let store = new_store();
    let entity = entity_type(kind);
    ensure_authorized_blocking(&store, entity)?;
    let reminders = kind == SyncProviderKind::AppleReminders;

    let changes = linked_ids
        .into_iter()
        .filter_map(|remote_id| verify_link(&store, reminders, remote_id))
        .collect();
    Ok(PullBatch {
        changes,
        next_cursor: None,
    })
}

/// The current remote state of one linked id. Returns `None` only when the
/// identifier resolves to the wrong item type for this provider's kind (e.g.
/// a reminder id on a calendar account) — skipped defensively rather than
/// misread as a deletion or a bogus snapshot.
fn verify_link(store: &EKEventStore, reminders: bool, remote_id: String) -> Option<RemoteChange> {
    let Some(existing) = fetch_item(store, &remote_id) else {
        // The identifier no longer resolves — deleted externally.
        return Some(RemoteChange {
            remote_id,
            etag: None,
            updated: None,
            deleted: true,
            item: None,
        });
    };
    let (updated, item) = if reminders {
        let reminder = existing.downcast::<EKReminder>().ok()?;
        (last_modified(&reminder), read_reminder(&reminder))
    } else {
        let event = existing.downcast::<EKEvent>().ok()?;
        (last_modified(&event), read_event(&event))
    };
    Some(RemoteChange {
        remote_id,
        etag: None,
        updated,
        deleted: false,
        item: Some(item),
    })
}

fn read_event(event: &EKEvent) -> RemoteItem {
    // SAFETY: Plain property getters on a valid EKEvent fetched from the
    // store; retained return values are converted before the block ends.
    unsafe {
        RemoteItem {
            title: event.title().to_string(),
            notes: event.notes().map(|n| n.to_string()),
            start: utc_from_ns_date(&event.startDate()),
            end: utc_from_ns_date(&event.endDate()),
            due: None,
            all_day: event.isAllDay(),
            completed: false,
            completed_at: None,
        }
    }
}

fn read_reminder(reminder: &EKReminder) -> RemoteItem {
    // SAFETY: Plain property getters on a valid EKReminder fetched from the
    // store; retained return values are converted before the block ends.
    unsafe {
        RemoteItem {
            title: reminder.title().to_string(),
            notes: reminder.notes().map(|n| n.to_string()),
            start: None,
            end: None,
            due: reminder
                .dueDateComponents()
                .as_deref()
                .and_then(due_from_components),
            all_day: false,
            completed: reminder.isCompleted(),
            completed_at: reminder
                .completionDate()
                .as_deref()
                .and_then(utc_from_ns_date),
        }
    }
}

/// Resolve a reminder's due-date components to a concrete instant via the
/// user's current calendar — the inverse of [`due_date_components`]. `None`
/// when the components do not name a resolvable date.
fn due_from_components(components: &NSDateComponents) -> Option<DateTime<Utc>> {
    NSCalendar::currentCalendar()
        .dateFromComponents(components)
        .as_deref()
        .and_then(utc_from_ns_date)
}

// ---------------------------------------------------------------------------
// Field mapping
// ---------------------------------------------------------------------------

fn apply_event_fields(event: &EKEvent, item: &RemoteItem) -> Result<(), String> {
    let start = item.start.ok_or("event push requires a start time")?;
    let title = NSString::from_str(&item.title);
    let notes = item.notes.as_deref().map(NSString::from_str);
    let (ns_start, ns_end) = event_window(start, item.end, item.all_day);

    // SAFETY: Plain property setters on a valid, store-bound EKEvent.
    unsafe {
        event.setTitle(Some(&title));
        event.setNotes(notes.as_deref());
        event.setAllDay(item.all_day);
        event.setStartDate(Some(&ns_start));
        event.setEndDate(Some(&ns_end));
    }
    Ok(())
}

/// Compute the event's date window. Timed events use start..end (or a default
/// 30-minute block); all-day events snap to the local day containing `start`.
fn event_window(
    start: DateTime<Utc>,
    end: Option<DateTime<Utc>>,
    all_day: bool,
) -> (Retained<NSDate>, Retained<NSDate>) {
    let ns_start = ns_date_from_utc(start);
    if all_day {
        let day_start = NSCalendar::currentCalendar().startOfDayForDate(&ns_start);
        let day_end = day_start.dateByAddingTimeInterval(ALL_DAY_WINDOW_SECS);
        return (day_start, day_end);
    }
    let ns_end = end
        .map(ns_date_from_utc)
        .unwrap_or_else(|| ns_start.dateByAddingTimeInterval(DEFAULT_EVENT_DURATION_SECS));
    (ns_start, ns_end)
}

fn apply_reminder_fields(reminder: &EKReminder, item: &RemoteItem) {
    let title = NSString::from_str(&item.title);
    let notes = item.notes.as_deref().map(NSString::from_str);
    let due = item.due.map(due_date_components);

    // SAFETY: Plain property setters on a valid, store-bound EKReminder.
    // setCompleted(true) stamps completionDate with "now"; an explicit
    // completed_at overrides it right after. setCompleted(false) clears it.
    unsafe {
        reminder.setTitle(Some(&title));
        reminder.setNotes(notes.as_deref());
        reminder.setDueDateComponents(due.as_deref());
        reminder.setCompleted(item.completed);
        if item.completed {
            if let Some(at) = item.completed_at {
                reminder.setCompletionDate(Some(&ns_date_from_utc(at)));
            }
        }
    }
}

/// Date-level due date (year/month/day in the user's current calendar) —
/// Apple Reminders treats hour-less components as an all-day due date.
fn due_date_components(due: DateTime<Utc>) -> Retained<NSDateComponents> {
    let ns_due = ns_date_from_utc(due);
    NSCalendar::currentCalendar().components_fromDate(
        NSCalendarUnit::Year | NSCalendarUnit::Month | NSCalendarUnit::Day,
        &ns_due,
    )
}

fn last_modified(item: &EKCalendarItem) -> Option<DateTime<Utc>> {
    // SAFETY: Property getter on a valid calendar item; nil for never-saved
    // items.
    unsafe { item.lastModifiedDate() }
        .as_deref()
        .and_then(utc_from_ns_date)
}

fn save_event(store: &EKEventStore, event: &EKEvent) -> Result<(), String> {
    // SAFETY: Saving an event created/fetched on this store; only this
    // occurrence, committing immediately.
    unsafe { store.saveEvent_span_commit_error(event, EKSpan::ThisEvent, true) }
        .map_err(|e| e.localizedDescription().to_string())
}

fn save_reminder(store: &EKEventStore, reminder: &EKReminder) -> Result<(), String> {
    // SAFETY: Saving a reminder created/fetched on this store, committing
    // immediately.
    unsafe { store.saveReminder_commit_error(reminder, true) }
        .map_err(|e| e.localizedDescription().to_string())
}

// ---------------------------------------------------------------------------
// Time conversion
// ---------------------------------------------------------------------------

fn ns_date_from_utc(dt: DateTime<Utc>) -> Retained<NSDate> {
    NSDate::dateWithTimeIntervalSince1970(dt.timestamp_millis() as f64 / 1000.0)
}

fn utc_from_ns_date(date: &NSDate) -> Option<DateTime<Utc>> {
    let millis = (date.timeIntervalSince1970() * 1000.0).round() as i64;
    Utc.timestamp_millis_opt(millis).single()
}
