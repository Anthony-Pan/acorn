//! Apple EventKit provider (macOS) — calendars and reminders via `EKEventStore`.
//!
//! NOTE: this is a compiling placeholder. The real objc2-event-kit bridge
//! (full-access request, EKEvent/EKReminder CRUD, change-notification pulls)
//! lands in the next commit. The struct + trait shape are fixed so the factory
//! and engine already target it.

use async_trait::async_trait;

use crate::sync::error::{SyncError, SyncResult};
use crate::sync::provider::SyncProvider;
use crate::sync::types::{
    PullBatch, PushOutcome, RemoteContainer, SyncAccount, SyncOp, SyncProviderKind,
};

pub struct AppleEventKitProvider {
    /// Only read via the trait's Phase 2 surface; the real bridge also uses it
    /// to pick the EKEntityType (event vs reminder).
    #[allow(dead_code)]
    kind: SyncProviderKind,
}

impl AppleEventKitProvider {
    pub fn new(account: &SyncAccount) -> SyncResult<Self> {
        let kind = account
            .kind()
            .filter(|k| k.is_apple())
            .ok_or_else(|| SyncError::Unavailable("not an Apple provider".into()))?;
        Ok(Self { kind })
    }
}

#[async_trait]
impl SyncProvider for AppleEventKitProvider {
    fn kind(&self) -> SyncProviderKind {
        self.kind
    }

    async fn list_containers(&self) -> SyncResult<Vec<RemoteContainer>> {
        Err(SyncError::NotImplemented("EventKit bridge pending".into()))
    }

    async fn push(&self, _ops: Vec<SyncOp>) -> SyncResult<Vec<PushOutcome>> {
        Err(SyncError::NotImplemented("EventKit bridge pending".into()))
    }

    async fn pull(&self, _cursor: Option<String>) -> SyncResult<PullBatch> {
        Err(SyncError::NotImplemented("EventKit bridge pending".into()))
    }

    async fn full_pull(&self) -> SyncResult<PullBatch> {
        Err(SyncError::NotImplemented("EventKit bridge pending".into()))
    }
}
