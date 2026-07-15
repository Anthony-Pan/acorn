use async_trait::async_trait;

use super::error::{SyncError, SyncResult};
use super::providers;
use super::types::{
    PullBatch, PushOutcome, RemoteContainer, SyncAccount, SyncOp, SyncProviderKind,
};

/// A remote calendar / reminders backend that Acorn reconciles tasks against.
/// One implementation per system; the engine drives them all uniformly.
#[async_trait]
pub trait SyncProvider: Send + Sync {
    /// Which backend this provider talks to. No call site until the Phase 2
    /// pull dispatch; kept so every impl declares its identity.
    #[allow(dead_code)]
    fn kind(&self) -> SyncProviderKind;

    /// Calendars / task lists the user can choose to sync into.
    async fn list_containers(&self) -> SyncResult<Vec<RemoteContainer>>;

    /// Apply a batch of create/update/delete ops. Per-item outcomes come back
    /// keyed by link id; a `PushResult::Failed`/`Conflict` is recorded by the
    /// engine rather than aborting the whole batch.
    async fn push(&self, ops: Vec<SyncOp>) -> SyncResult<Vec<PushOutcome>>;

    /// Incremental pull from `cursor` (Phase 2+; the trait fixes the shape now
    /// so providers don't churn). Returning [`SyncError::FullResyncRequired`]
    /// tells the engine to fall back to [`SyncProvider::full_pull`].
    #[allow(dead_code)]
    async fn pull(&self, cursor: Option<String>) -> SyncResult<PullBatch>;

    /// Full pull — first sync and 410 recovery (Phase 2+).
    #[allow(dead_code)]
    async fn full_pull(&self) -> SyncResult<PullBatch>;
}

/// Construct the provider for an account, minting a fresh access token from the
/// stored refresh token for Google. Mirrors `ai::build_provider`'s match-on-kind
/// factory shape.
pub async fn build_provider(account: &SyncAccount) -> SyncResult<Box<dyn SyncProvider>> {
    let kind = account.kind().ok_or_else(|| {
        SyncError::Unavailable(format!("unknown provider '{}'", account.provider))
    })?;
    match kind {
        SyncProviderKind::GoogleCalendar => Ok(Box::new(
            providers::google_calendar::GoogleCalendarProvider::connect(account).await?,
        )),
        SyncProviderKind::GoogleTasks => Ok(Box::new(
            providers::google_tasks::GoogleTasksProvider::connect(account).await?,
        )),
        SyncProviderKind::AppleCalendar | SyncProviderKind::AppleReminders => {
            providers::apple::build(account)
        }
    }
}
