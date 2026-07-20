//! Pure task ↔ remote-item projection and content hashing. Kept free of DB and
//! network so it can be unit-tested directly.

use sha2::{Digest, Sha256};

use crate::db::models::{Task, TaskStatus};

use super::types::{RemoteItem, TargetKind};

/// Project a task onto a provider-agnostic remote item for the given target.
///
/// Event target: use the schedule window (falling back to `due_date` for a
/// start). Reminder target: use the due date (falling back to `scheduled_start`).
pub fn map_task(task: &Task, target: TargetKind) -> RemoteItem {
    let completed = matches!(task.status, TaskStatus::Completed);
    let notes = task.description.clone();
    match target {
        TargetKind::Event => RemoteItem {
            title: task.title.clone(),
            notes,
            start: task.scheduled_start.or(task.due_date),
            end: task.scheduled_end,
            due: None,
            all_day: task.all_day,
            completed,
            completed_at: task.completed_at,
        },
        TargetKind::Reminder => RemoteItem {
            title: task.title.clone(),
            notes,
            start: None,
            end: None,
            due: task.due_date.or(task.scheduled_start),
            all_day: false,
            completed,
            completed_at: task.completed_at,
        },
    }
}

/// A stable hash of the fields Acorn pushes. Two items with the same hash are
/// content-identical, which lets the engine skip no-op updates and (in Phase 2)
/// recognise the echo of its own writes on pull.
pub fn content_hash(item: &RemoteItem) -> String {
    let mut hasher = Sha256::new();
    let unit = '\u{1f}'; // unit separator — unambiguous field boundary
    let canonical = format!(
        "{}{unit}{}{unit}{}{unit}{}{unit}{}{unit}{}{unit}{}",
        item.title,
        item.notes.as_deref().unwrap_or(""),
        item.start.map(|d| d.to_rfc3339()).unwrap_or_default(),
        item.end.map(|d| d.to_rfc3339()).unwrap_or_default(),
        item.due.map(|d| d.to_rfc3339()).unwrap_or_default(),
        item.all_day,
        item.completed,
    );
    hasher.update(canonical.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::db::models::Priority;

    fn task() -> Task {
        Task {
            id: "t1".into(),
            session_id: "s1".into(),
            title: "Write the report".into(),
            description: Some("Quarterly numbers".into()),
            duration_minutes: 45,
            priority: Priority::Medium,
            order_index: 0,
            status: TaskStatus::Pending,
            created_at: Utc.with_ymd_and_hms(2026, 7, 1, 9, 0, 0).unwrap(),
            started_at: None,
            completed_at: None,
            scheduled_start: Some(Utc.with_ymd_and_hms(2026, 7, 2, 14, 0, 0).unwrap()),
            scheduled_end: Some(Utc.with_ymd_and_hms(2026, 7, 2, 15, 0, 0).unwrap()),
            due_date: Some(Utc.with_ymd_and_hms(2026, 7, 3, 0, 0, 0).unwrap()),
            all_day: false,
            updated_at: Utc.with_ymd_and_hms(2026, 7, 1, 9, 0, 0).unwrap(),
        }
    }

    #[test]
    fn event_target_uses_schedule_window() {
        let item = map_task(&task(), TargetKind::Event);
        assert_eq!(item.title, "Write the report");
        assert_eq!(item.notes.as_deref(), Some("Quarterly numbers"));
        assert_eq!(item.start, task().scheduled_start);
        assert_eq!(item.end, task().scheduled_end);
        assert_eq!(item.due, None);
        assert!(!item.completed);
    }

    #[test]
    fn event_target_falls_back_to_due_date_for_start() {
        let mut t = task();
        t.scheduled_start = None;
        t.scheduled_end = None;
        let item = map_task(&t, TargetKind::Event);
        assert_eq!(item.start, t.due_date);
        assert_eq!(item.end, None);
    }

    #[test]
    fn reminder_target_uses_due_date_and_ignores_window() {
        let item = map_task(&task(), TargetKind::Reminder);
        assert_eq!(item.due, task().due_date);
        assert_eq!(item.start, None);
        assert_eq!(item.end, None);
        assert!(!item.all_day);
    }

    #[test]
    fn reminder_target_falls_back_to_scheduled_start_for_due() {
        let mut t = task();
        t.due_date = None;
        let item = map_task(&t, TargetKind::Reminder);
        assert_eq!(item.due, t.scheduled_start);
    }

    #[test]
    fn completed_status_maps_to_done() {
        let mut t = task();
        t.status = TaskStatus::Completed;
        t.completed_at = Some(Utc.with_ymd_and_hms(2026, 7, 2, 16, 0, 0).unwrap());
        let item = map_task(&t, TargetKind::Reminder);
        assert!(item.completed);
        assert_eq!(item.completed_at, t.completed_at);
        // Skipped is not "done" externally — it stays an open item.
        t.status = TaskStatus::Skipped;
        assert!(!map_task(&t, TargetKind::Reminder).completed);
    }

    #[test]
    fn identical_content_hashes_identically() {
        let a = map_task(&task(), TargetKind::Event);
        let b = map_task(&task(), TargetKind::Event);
        assert_eq!(content_hash(&a), content_hash(&b));
    }

    #[test]
    fn any_synced_field_change_alters_the_hash() {
        let base = map_task(&task(), TargetKind::Event);
        let base_hash = content_hash(&base);

        let mut retitled = base.clone();
        retitled.title = "Write the report v2".into();
        assert_ne!(content_hash(&retitled), base_hash);

        let mut rescheduled = base.clone();
        rescheduled.start = Some(Utc.with_ymd_and_hms(2026, 7, 2, 14, 30, 0).unwrap());
        assert_ne!(content_hash(&rescheduled), base_hash);

        let mut done = base.clone();
        done.completed = true;
        assert_ne!(content_hash(&done), base_hash);
    }

    #[test]
    fn field_boundaries_do_not_collide() {
        // "ab" + "c" must not hash like "a" + "bc" — the separator guards this.
        let mut left = RemoteItem {
            title: "ab".into(),
            notes: Some("c".into()),
            ..RemoteItem::default()
        };
        let right = RemoteItem {
            title: "a".into(),
            notes: Some("bc".into()),
            ..RemoteItem::default()
        };
        assert_ne!(content_hash(&left), content_hash(&right));
        // Sanity: equal content still collides deliberately.
        left.title = "a".into();
        left.notes = Some("bc".into());
        assert_eq!(content_hash(&left), content_hash(&right));
    }
}
