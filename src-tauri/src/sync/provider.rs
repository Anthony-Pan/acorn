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
    /// Which backend this provider talks to.
    #[allow(dead_code)]
    fn kind(&self) -> SyncProviderKind;

    /// Calendars / task lists the user can choose to sync into.
    async fn list_containers(&self) -> SyncResult<Vec<RemoteContainer>>;

    /// Apply a batch of create/update/delete ops. Per-item outcomes come back
    /// keyed by link id; a `PushResult::Failed`/`Conflict` is recorded by the
    /// engine rather than aborting the whole batch.
    async fn push(&self, ops: Vec<SyncOp>) -> SyncResult<Vec<PushOutcome>>;

    /// Incremental pull from `cursor`. `linked_ids` are the remote ids Acorn
    /// currently tracks — Google impls ignore them (their change feeds are
    /// account-wide); EventKit verifies each id directly since the classic API
    /// has no change feed. Returning [`SyncError::FullResyncRequired`] tells
    /// the engine to clear the cursor and fall back to
    /// [`SyncProvider::full_pull`].
    async fn pull(&self, cursor: Option<String>, linked_ids: Vec<String>) -> SyncResult<PullBatch>;

    /// Full pull — first sync and 410 recovery.
    async fn full_pull(&self, linked_ids: Vec<String>) -> SyncResult<PullBatch>;
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
