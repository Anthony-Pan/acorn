use std::collections::HashMap;
use std::sync::Mutex;

use tauri::State;
use tokio::sync::oneshot;

use crate::error::{AppError, AppResult};

#[derive(Default)]
pub struct ApprovalBroker {
    pending: Mutex<HashMap<String, oneshot::Sender<bool>>>,
}

impl ApprovalBroker {
    pub fn register(&self, request_id: String) -> oneshot::Receiver<bool> {
        let (tx, rx) = oneshot::channel();
        if let Ok(mut guard) = self.pending.lock() {
            guard.insert(request_id, tx);
        }
        rx
    }

    pub fn resolve(&self, request_id: &str, allow: bool) -> bool {
        let sender = self
            .pending
            .lock()
            .ok()
            .and_then(|mut g| g.remove(request_id));
        if let Some(tx) = sender {
            let _ = tx.send(allow);
            true
        } else {
            false
        }
    }

    pub fn forget(&self, request_id: &str) {
        if let Ok(mut guard) = self.pending.lock() {
            guard.remove(request_id);
        }
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn respond_tool_approval(
    broker: State<'_, ApprovalBroker>,
    request_id: String,
    allow: bool,
) -> AppResult<()> {
    if !broker.resolve(&request_id, allow) {
        return Err(AppError::NotFound(format!(
            "no pending approval request for id {request_id}"
        )));
    }
    Ok(())
}
