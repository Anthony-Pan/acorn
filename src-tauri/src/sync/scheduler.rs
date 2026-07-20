//! Background sync loop with three wake sources: a base interval, a debounced
//! on-change signal pinged by the task write path (edit → external calendars in
//! seconds, not minutes), and — on macOS — the `EKEventStoreChanged`
//! notification so external Calendar/Reminders edits pull back promptly. Cycle
//! errors back the interval off exponentially (up to an hour); a success snaps
//! it back. The whole loop sits behind a `sync-disabled` settings kill switch.

use std::sync::Arc;

use tokio::sync::Notify;

/// Cross-platform wake signal for the sync loop. Managed in Tauri state so the
/// task write path (and account connects) can nudge a sync without knowing
/// anything about scheduling. A ping while a cycle runs is kept as a permit,
/// so changes made mid-cycle trigger a follow-up cycle rather than being lost.
#[derive(Clone, Default)]
pub struct SyncSignal {
    notify: Arc<Notify>,
}

impl SyncSignal {
    pub fn ping(&self) {
        self.notify.notify_one();
    }

    #[cfg(desktop)]
    fn notified(&self) -> tokio::sync::futures::Notified<'_> {
        self.notify.notified()
    }
}

#[cfg(desktop)]
mod run {
    use std::time::Duration;

    use tauri::Manager;

    use super::SyncSignal;

    /// Base cadence between unprompted cycles.
    const BASE_INTERVAL: Duration = Duration::from_secs(600);
    /// Ceiling for the error backoff.
    const MAX_INTERVAL: Duration = Duration::from_secs(3600);
    /// Give the app a quiet start before the first cycle.
    const INITIAL_DELAY: Duration = Duration::from_secs(30);
    /// Coalesce bursts of pings (rapid task edits, EventKit notification
    /// storms — it fires several times per change, including for our own
    /// writes) into one cycle.
    const DEBOUNCE: Duration = Duration::from_secs(3);

    /// Spawn the background sync task. Desktop-only, consistent with the tray
    /// and global-shortcut gating.
    pub fn spawn(app: tauri::AppHandle) {
        let database = app.state::<crate::db::Database>().inner().clone();
        let signal = app.state::<SyncSignal>().inner().clone();

        #[cfg(target_os = "macos")]
        super::eventkit_observer::register(signal.clone());

        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(INITIAL_DELAY).await;
            let mut interval = BASE_INTERVAL;
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(interval) => {}
                    _ = signal.notified() => {
                        tokio::time::sleep(DEBOUNCE).await;
                    }
                }
                if sync_disabled(&database).await {
                    continue;
                }
                let results = crate::sync::engine::sync_all_enabled(&database).await;
                let any_error = results.iter().any(|(_, result)| result.is_err());
                interval = if any_error {
                    (interval * 2).min(MAX_INTERVAL)
                } else {
                    BASE_INTERVAL
                };
            }
        });
    }

    async fn sync_disabled(db: &crate::db::Database) -> bool {
        matches!(
            crate::commands::settings::load_setting(db, "sync-disabled")
                .await
                .ok()
                .flatten()
                .as_deref(),
            Some("true")
        )
    }
}

#[cfg(desktop)]
pub use run::spawn;

/// External-change wakeups: a long-lived `EKEventStore` plus an
/// `EKEventStoreChanged` observer whose block pings the [`SyncSignal`]. The
/// store and observer token are intentionally leaked — they must outlive the
/// app or the notifications stop.
#[cfg(all(desktop, target_os = "macos"))]
mod eventkit_observer {
    use block2::RcBlock;
    use objc2_event_kit::{EKEventStore, EKEventStoreChangedNotification};
    use objc2_foundation::NSNotificationCenter;

    use super::SyncSignal;

    pub fn register(signal: SyncSignal) {
        // EKEventStore and the observer registration are cheap and
        // thread-agnostic, but `Retained<_>` is !Send — build and leak both on
        // a blocking thread so nothing objc2 crosses an await point.
        tauri::async_runtime::spawn_blocking(move || {
            // SAFETY: Plain no-argument constructor; EKEventStore may be
            // created on any thread. The store is leaked below so the change
            // notifications keep flowing for the app's lifetime.
            let store = unsafe { EKEventStore::new() };

            let block = RcBlock::new(move |_notification: std::ptr::NonNull<_>| {
                signal.ping();
            });

            // SAFETY: `defaultCenter` is a thread-safe singleton. The block
            // only captures a Send + Sync SyncSignal and is safe to invoke
            // from whatever thread EventKit posts on. `object: None` observes
            // the notification regardless of which store instance posted it.
            let token = unsafe {
                NSNotificationCenter::defaultCenter().addObserverForName_object_queue_usingBlock(
                    Some(EKEventStoreChangedNotification),
                    None,
                    None,
                    &block,
                )
            };

            // Keep the store and the observer token alive forever: dropping
            // the token deregisters the observer; dropping the store stops it
            // from watching the database.
            std::mem::forget(store);
            std::mem::forget(token);
        });
    }
}
