pub mod google_calendar;
pub mod google_http;
pub mod google_oauth;
pub mod google_tasks;

#[cfg(target_os = "macos")]
pub mod apple_eventkit;

/// Apple EventKit factory, gated by platform. EventKit is macOS-only, so on
/// other platforms the provider is simply unavailable.
pub mod apple {
    use crate::sync::error::SyncResult;
    use crate::sync::provider::SyncProvider;
    use crate::sync::types::SyncAccount;

    #[cfg(target_os = "macos")]
    pub fn build(account: &SyncAccount) -> SyncResult<Box<dyn SyncProvider>> {
        Ok(Box::new(super::apple_eventkit::AppleEventKitProvider::new(
            account,
        )?))
    }

    #[cfg(not(target_os = "macos"))]
    pub fn build(_account: &SyncAccount) -> SyncResult<Box<dyn SyncProvider>> {
        Err(crate::sync::error::SyncError::Unavailable(
            "Apple Calendar/Reminders sync requires macOS".into(),
        ))
    }
}
