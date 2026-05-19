use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::mpsc;

use super::chat::{ChatMessage, ChatTurn};
use super::error::{ProviderError, ProviderResult};
use super::metadata::{ApiFormat, ProviderMetadata};
use super::providers::{
    acorn_cloud::AcornCloudProvider,
    anthropic::AnthropicProvider,
    cli::{invocation_for, CliProvider},
    ollama::OllamaProvider,
    openai_compatible::OpenAiCompatibleProvider,
};
use super::tools::ToolSpec;
use super::types::{DecomposeEvent, DecomposeRequest, DecomposeResponse};

#[async_trait]
pub trait Provider: Send + Sync {
    fn metadata(&self) -> &ProviderMetadata;

    async fn decompose(
        &self,
        request: DecomposeRequest,
        events: mpsc::Sender<DecomposeEvent>,
    ) -> ProviderResult<DecomposeResponse>;

    async fn validate_credentials(&self) -> ProviderResult<()>;

    async fn chat_turn(
        &self,
        messages: &[ChatMessage],
        tools: &[ToolSpec],
        system_prompt: &str,
    ) -> ProviderResult<ChatTurn>;

    fn supports_tools(&self) -> bool {
        true
    }
}

pub struct ProviderInputs {
    pub metadata: ProviderMetadata,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub custom_endpoint: Option<String>,
}

pub fn build_provider(inputs: ProviderInputs) -> ProviderResult<Box<dyn Provider>> {
    let ProviderInputs {
        metadata,
        api_key,
        model,
        custom_endpoint,
    } = inputs;
    let model = model.unwrap_or_else(|| metadata.default_model.to_string());
    let api_key = api_key.unwrap_or_default();

    match metadata.api_format {
        ApiFormat::Anthropic => Ok(Box::new(AnthropicProvider::new(
            metadata,
            api_key,
            model,
            custom_endpoint,
        ))),
        ApiFormat::OpenAiCompatible => Ok(Box::new(OpenAiCompatibleProvider::new(
            metadata,
            api_key,
            model,
            custom_endpoint,
        ))),
        ApiFormat::Ollama => Ok(Box::new(OllamaProvider::new(
            metadata,
            model,
            custom_endpoint,
        ))),
        ApiFormat::AcornCloud => Ok(Box::new(AcornCloudProvider::new(metadata))),
        ApiFormat::Gemini => Err(ProviderError::NotImplemented(
            "Gemini provider lands in v1.1".into(),
        )),
        ApiFormat::Cli => {
            let invocation = invocation_for(metadata.id).ok_or_else(|| {
                ProviderError::NotImplemented(format!(
                    "no CLI invocation registered for provider id {}",
                    metadata.id
                ))
            })?;
            Ok(Box::new(CliProvider::new(metadata, invocation)))
        }
    }
}

pub fn strip_code_fences(text: &str) -> &str {
    let trimmed = text.trim();
    for prefix in ["```json", "```JSON", "```"] {
        if let Some(after) = trimmed.strip_prefix(prefix) {
            let after = after.trim_start_matches('\n').trim_start();
            if let Some((body, _)) = after.rsplit_once("```") {
                return body.trim();
            }
        }
    }
    trimmed
}

pub async fn emit_decomposed(
    events: &mpsc::Sender<DecomposeEvent>,
    response: &DecomposeResponse,
) -> ProviderResult<()> {
    for task in &response.tasks {
        events
            .send(DecomposeEvent::Task { task: task.clone() })
            .await?;
        tokio::time::sleep(Duration::from_millis(120)).await;
    }
    events
        .send(DecomposeEvent::Summary {
            summary: response.summary.clone(),
        })
        .await?;
    events.send(DecomposeEvent::Done).await?;
    Ok(())
}

pub fn format_user_input(request: &DecomposeRequest) -> String {
    let mut input = format!("Language: {}\n\n", request.language);
    if let Some(ctx) = &request.user_context {
        input.push_str("Context: ");
        input.push_str(ctx);
        input.push_str("\n\n");
    }
    input.push_str("User input:\n");
    input.push_str(&request.raw_input);
    input
}

pub fn parse_decompose(raw: &str) -> ProviderResult<DecomposeResponse> {
    use super::types::{DecomposedSubtask, DecomposedTask};
    use crate::db::models::Priority;
    use serde::Deserialize;
    use uuid::Uuid;

    #[derive(Debug, Deserialize)]
    struct LenientResponse {
        #[serde(default)]
        tasks: Vec<LenientTask>,
        #[serde(default)]
        summary: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    struct LenientTask {
        #[serde(default)]
        id: Option<String>,
        #[serde(default, alias = "name", alias = "task")]
        title: serde_json::Value,
        #[serde(default)]
        description: Option<String>,
        #[serde(
            default,
            alias = "duration",
            alias = "minutes",
            alias = "time",
            alias = "estimated_minutes"
        )]
        duration_minutes: Option<serde_json::Value>,
        #[serde(default, alias = "priority_level", alias = "urgency")]
        priority: Option<String>,
        #[serde(default, alias = "index", alias = "position")]
        order: Option<u32>,
        #[serde(default, alias = "steps")]
        subtasks: Vec<LenientSubtask>,
    }

    #[derive(Debug, Deserialize)]
    struct LenientSubtask {
        #[serde(default, alias = "name", alias = "step")]
        title: serde_json::Value,
        #[serde(default)]
        done: bool,
    }

    fn value_to_string(v: &serde_json::Value) -> Option<String> {
        match v {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            serde_json::Value::Object(map) => map
                .get("title")
                .or_else(|| map.get("text"))
                .or_else(|| map.get("name"))
                .and_then(|v| v.as_str())
                .map(String::from),
            _ => None,
        }
    }

    fn coerce_number(v: &serde_json::Value) -> Option<u32> {
        match v {
            serde_json::Value::Number(n) => n.as_u64().map(|x| x as u32),
            serde_json::Value::String(s) => s
                .trim()
                .trim_end_matches(|c: char| !c.is_ascii_digit())
                .parse::<u32>()
                .ok(),
            _ => None,
        }
    }

    fn lenient_into_strict(task: LenientTask, fallback_order: u32) -> Option<DecomposedTask> {
        let title = value_to_string(&task.title)?;
        if title.trim().is_empty() {
            return None;
        }
        let duration_minutes = task
            .duration_minutes
            .as_ref()
            .and_then(coerce_number)
            .unwrap_or(15)
            .clamp(1, 8 * 60);
        let priority = match task.priority.as_deref().map(str::to_lowercase).as_deref() {
            Some("high") | Some("urgent") | Some("critical") => Priority::High,
            Some("low") | Some("optional") => Priority::Low,
            _ => Priority::Medium,
        };
        let subtasks = task
            .subtasks
            .into_iter()
            .filter_map(|s| {
                value_to_string(&s.title).map(|t| DecomposedSubtask {
                    title: t.trim().to_string(),
                    done: s.done,
                })
            })
            .filter(|s| !s.title.is_empty())
            .collect();
        Some(DecomposedTask {
            id: task.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            title: title.trim().to_string(),
            description: task.description.filter(|s| !s.trim().is_empty()),
            duration_minutes,
            priority,
            order: task.order.unwrap_or(fallback_order),
            subtasks,
        })
    }

    let stripped = strip_code_fences(raw);
    let json_str = {
        let start = stripped.find('{');
        let end = stripped.rfind('}');
        match (start, end) {
            (Some(s), Some(e)) if e > s => &stripped[s..=e],
            _ => stripped,
        }
    };

    if let Ok(strict) = serde_json::from_str::<DecomposeResponse>(json_str) {
        return Ok(strict);
    }

    let lenient: LenientResponse = serde_json::from_str(json_str).map_err(|e| {
        let preview = if json_str.len() <= 200 {
            json_str.trim().to_string()
        } else {
            format!("{}…", &json_str.trim()[..200])
        };
        ProviderError::InvalidResponse(format!(
            "could not parse model output as a task plan: {e}. Got: {preview}"
        ))
    })?;

    let tasks = lenient
        .tasks
        .into_iter()
        .enumerate()
        .filter_map(|(i, t)| lenient_into_strict(t, i as u32))
        .collect();

    Ok(DecomposeResponse {
        tasks,
        summary: lenient
            .summary
            .unwrap_or_else(|| "Today's plan.".to_string()),
    })
}
