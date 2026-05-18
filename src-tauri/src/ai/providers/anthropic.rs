use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::ai::chat::{ChatMessage, ChatRole, ChatTurn, ToolCall};
use crate::ai::error::{ProviderError, ProviderResult};
use crate::ai::metadata::ProviderMetadata;
use crate::ai::prompts::DECOMPOSE_SYSTEM_PROMPT;
use crate::ai::provider::{emit_decomposed, format_user_input, strip_code_fences, Provider};
use crate::ai::tools::ToolSpec;
use crate::ai::types::{DecomposeEvent, DecomposeRequest, DecomposeResponse};

const ANTHROPIC_VERSION: &str = "2023-06-01";
const MAX_TOKENS: u32 = 2048;

pub struct AnthropicProvider {
    metadata: ProviderMetadata,
    api_key: String,
    model: String,
    endpoint: String,
    client: Client,
}

impl AnthropicProvider {
    pub fn new(
        metadata: ProviderMetadata,
        api_key: String,
        model: String,
        custom_endpoint: Option<String>,
    ) -> Self {
        let endpoint = custom_endpoint.unwrap_or_else(|| metadata.default_endpoint.to_string());
        Self {
            metadata,
            api_key,
            model,
            endpoint,
            client: Client::new(),
        }
    }

    fn require_key(&self) -> ProviderResult<&str> {
        if self.api_key.is_empty() {
            return Err(ProviderError::MissingCredentials(
                self.metadata.id.to_string(),
            ));
        }
        Ok(self.api_key.as_str())
    }
}

#[derive(Debug, Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    system: &'a str,
    messages: Vec<AnthropicMessage<'a>>,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicContent {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(other)]
    Other,
}

#[async_trait]
impl Provider for AnthropicProvider {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn decompose(
        &self,
        request: DecomposeRequest,
        events: mpsc::Sender<DecomposeEvent>,
    ) -> ProviderResult<DecomposeResponse> {
        let api_key = self.require_key()?;
        let user_content = format_user_input(&request);

        let body = AnthropicRequest {
            model: &self.model,
            max_tokens: MAX_TOKENS,
            system: DECOMPOSE_SYSTEM_PROMPT,
            messages: vec![AnthropicMessage {
                role: "user",
                content: &user_content,
            }],
            temperature: 0.3,
        };

        let _ = events
            .send(DecomposeEvent::Progress { received_chars: 0 })
            .await;

        let resp = self
            .client
            .post(&self.endpoint)
            .header("x-api-key", api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ProviderError::InvalidKey);
        }
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ProviderError::ProviderResponse(format!(
                "HTTP {status}: {text}"
            )));
        }

        let parsed: AnthropicResponse = resp.json().await?;
        let text = parsed
            .content
            .into_iter()
            .find_map(|c| match c {
                AnthropicContent::Text { text } => Some(text),
                _ => None,
            })
            .ok_or_else(|| ProviderError::InvalidResponse("no text content".into()))?;

        let _ = events
            .send(DecomposeEvent::Progress {
                received_chars: text.len(),
            })
            .await;

        let json = strip_code_fences(&text);
        let response: DecomposeResponse = serde_json::from_str(json)?;

        emit_decomposed(&events, &response).await?;
        Ok(response)
    }

    async fn validate_credentials(&self) -> ProviderResult<()> {
        let api_key = self.require_key()?;

        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": 1,
            "messages": [{ "role": "user", "content": "." }],
        });

        let resp = self
            .client
            .post(&self.endpoint)
            .header("x-api-key", api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ProviderError::InvalidKey);
        }
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ProviderError::ProviderResponse(text));
        }
        Ok(())
    }

    async fn chat_turn(
        &self,
        messages: &[ChatMessage],
        tools: &[ToolSpec],
        system_prompt: &str,
    ) -> ProviderResult<ChatTurn> {
        let api_key = self.require_key()?;
        let anthropic_messages = to_anthropic_messages(messages);
        let anthropic_tools: Vec<serde_json::Value> = tools
            .iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "description": t.description,
                    "input_schema": t.parameters,
                })
            })
            .collect();

        let mut body = serde_json::json!({
            "model": self.model,
            "max_tokens": MAX_TOKENS,
            "system": system_prompt,
            "messages": anthropic_messages,
        });
        if !anthropic_tools.is_empty() {
            body["tools"] = serde_json::Value::Array(anthropic_tools);
        }

        let resp = self
            .client
            .post(&self.endpoint)
            .header("x-api-key", api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ProviderError::InvalidKey);
        }
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ProviderError::ProviderResponse(format!(
                "HTTP {status}: {text}"
            )));
        }

        let parsed: AnthropicResponse = resp.json().await?;
        let mut text = String::new();
        let mut tool_calls = Vec::new();
        for block in parsed.content {
            match block {
                AnthropicContent::Text { text: t } => text.push_str(&t),
                AnthropicContent::ToolUse { id, name, input } => {
                    tool_calls.push(ToolCall {
                        id,
                        name,
                        arguments: input,
                    });
                }
                AnthropicContent::Other => {}
            }
        }
        Ok(ChatTurn { text, tool_calls })
    }
}

fn to_anthropic_messages(messages: &[ChatMessage]) -> Vec<serde_json::Value> {
    let mut out = Vec::with_capacity(messages.len());
    for msg in messages {
        match msg.role {
            ChatRole::System => {}
            ChatRole::User => out.push(serde_json::json!({
                "role": "user",
                "content": msg.content,
            })),
            ChatRole::Assistant => {
                let mut blocks: Vec<serde_json::Value> = Vec::new();
                if !msg.content.is_empty() {
                    blocks.push(serde_json::json!({ "type": "text", "text": msg.content }));
                }
                for call in &msg.tool_calls {
                    blocks.push(serde_json::json!({
                        "type": "tool_use",
                        "id": call.id,
                        "name": call.name,
                        "input": call.arguments,
                    }));
                }
                out.push(serde_json::json!({ "role": "assistant", "content": blocks }));
            }
            ChatRole::Tool => {
                let tool_use_id = msg.tool_call_id.clone().unwrap_or_default();
                out.push(serde_json::json!({
                    "role": "user",
                    "content": [{
                        "type": "tool_result",
                        "tool_use_id": tool_use_id,
                        "content": msg.content,
                    }],
                }));
            }
        }
    }
    out
}
