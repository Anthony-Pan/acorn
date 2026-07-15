//! Background sync loop. Phase 1 is a plain interval poll of every enabled
//! account, gated behind a `sync-disabled` setting kill switch. On-change
//! debounce triggers and EventKit change-notification wakeups arrive in Phase 4.

#[cfg(desktop)]
use std::time::Duration;

#[cfg(desktop)]
const INITIAL_DELAY: Duration = Duration::from_secs(30);
#[cfg(desktop)]
const INTERVAL: Duration = Duration::from_secs(600);

/// Spawn the background sync task. Desktop-only, consistent with the tray and
/// global-shortcut gating.
#[cfg(desktop)]
pub fn spawn(app: tauri::AppHandle) {
    use tauri::Manager;

    let database = app.state::<crate::db::Database>().inner().clone();
    tauri::async_runtime::spawn(async move {
        // Don't compete with app startup.
        tokio::time::sleep(INITIAL_DELAY).await;
        loop {
            if !sync_disabled(&database).await {
                let _ = super::engine::sync_all_enabled(&database).await;
            }
            tokio::time::sleep(INTERVAL).await;
        }
    });
}

#[cfg(desktop)]
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
