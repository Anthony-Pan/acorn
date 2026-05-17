use serde::{Deserialize, Serialize};

use crate::db::models::Priority;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecomposeRequest {
    pub raw_input: String,
    pub language: String,
    #[serde(default)]
    pub user_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecomposedTask {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub duration_minutes: u32,
    pub priority: Priority,
    pub order: u32,
    #[serde(default)]
    pub subtasks: Vec<DecomposedSubtask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecomposedSubtask {
    pub title: String,
    #[serde(default)]
    pub done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecomposeResponse {
    pub tasks: Vec<DecomposedTask>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum DecomposeEvent {
    Progress { received_chars: usize },
    Task { task: DecomposedTask },
    Summary { summary: String },
    Done,
}
