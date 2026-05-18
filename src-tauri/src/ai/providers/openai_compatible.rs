use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::ai::chat::{ChatMessage, ChatRole, ChatTurn, ToolCall};
use crate::ai::error::{ProviderError, ProviderResult};
use crate::ai::metadata::ProviderMetadata;
use crate::ai::prompts::DECOMPOSE_SYSTEM_PROMPT;
use crate::ai::provider::{emit_decomposed, format_user_input, parse_decompose, Provider};
use crate::ai::tools::ToolSpec;
use crate::ai::types::{DecomposeEvent, DecomposeRequest, DecomposeResponse};

pub struct OpenAiCompatibleProvider {
    metadata: ProviderMetadata,
    api_key: String,
    model: String,
    endpoint: String,
    client: Client,
}

impl OpenAiCompatibleProvider {
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
struct DecomposeRequestBody<'a> {
    model: &'a str,
    messages: Vec<WireMessage<'a>>,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
}

#[derive(Debug, Serialize)]
struct WireMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum ResponseFormat {
    #[serde(rename = "json_object")]
    JsonObject,
}

#[derive(Debug, Deserialize)]
struct DecomposeResponseBody {
    choices: Vec<DecomposeChoice>,
}

#[derive(Debug, Deserialize)]
struct DecomposeChoice {
    message: DecomposeChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct DecomposeChoiceMessage {
    content: String,
}

fn supports_json_mode(provider_id: &str) -> bool {
    matches!(
        provider_id,
        "openai" | "deepseek" | "openrouter" | "groq" | "mistral" | "moonshot" | "qwen"
    )
}

#[async_trait]
impl Provider for OpenAiCompatibleProvider {
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

        let body = DecomposeRequestBody {
            model: &self.model,
            messages: vec![
                WireMessage {
                    role: "system",
                    content: DECOMPOSE_SYSTEM_PROMPT,
                },
                WireMessage {
                    role: "user",
                    content: &user_content,
                },
            ],
            temperature: 0.3,
            response_format: supports_json_mode(self.metadata.id)
                .then_some(ResponseFormat::JsonObject),
        };

        let _ = events
            .send(DecomposeEvent::Progress { received_chars: 0 })
            .await;

        let resp = self
            .client
            .post(&self.endpoint)
            .bearer_auth(api_key)
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

        let parsed: DecomposeResponseBody = resp.json().await?;
        let content = parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| ProviderError::InvalidResponse("no choices in response".into()))?;

        let _ = events
            .send(DecomposeEvent::Progress {
                received_chars: content.len(),
            })
            .await;

        let response = parse_decompose(&content)?;

        emit_decomposed(&events, &response).await?;
        Ok(response)
    }

    async fn validate_credentials(&self) -> ProviderResult<()> {
        let api_key = self.require_key()?;

        let body = serde_json::json!({
            "model": self.model,
            "messages": [{ "role": "user", "content": "." }],
            "max_tokens": 1,
        });

        let resp = self
            .client
            .post(&self.endpoint)
            .bearer_auth(api_key)
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
        let mut openai_messages = vec![serde_json::json!({
            "role": "system",
            "content": system_prompt,
        })];
        openai_messages.extend(to_openai_messages(messages));

        let openai_tools: Vec<serde_json::Value> = tools
            .iter()
            .map(|t| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.parameters,
                    },
                })
            })
            .collect();

        let mut body = serde_json::json!({
            "model": self.model,
            "messages": openai_messages,
            "temperature": 0.4,
        });
        if !openai_tools.is_empty() {
            body["tools"] = serde_json::Value::Array(openai_tools);
        }

        let resp = self
            .client
            .post(&self.endpoint)
            .bearer_auth(api_key)
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

        let parsed: ChatTurnResponse = resp.json().await?;
        let choice = parsed
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| ProviderError::InvalidResponse("no choices in response".into()))?;

        let text = choice.message.content.unwrap_or_default();
        let tool_calls = choice
            .message
            .tool_calls
            .unwrap_or_default()
            .into_iter()
            .filter_map(|tc| {
                let arguments = serde_json::from_str(&tc.function.arguments).ok()?;
                Some(ToolCall {
                    id: tc.id,
                    name: tc.function.name,
                    arguments,
                })
            })
            .collect();

        Ok(ChatTurn { text, tool_calls })
    }
}

fn to_openai_messages(messages: &[ChatMessage]) -> Vec<serde_json::Value> {
    messages
        .iter()
        .map(|msg| match msg.role {
            ChatRole::System => serde_json::json!({ "role": "system", "content": msg.content }),
            ChatRole::User => serde_json::json!({ "role": "user", "content": msg.content }),
            ChatRole::Assistant => {
                let mut value = serde_json::json!({
                    "role": "assistant",
                    "content": if msg.content.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(msg.content.clone()) },
                });
                if !msg.tool_calls.is_empty() {
                    let tool_calls: Vec<serde_json::Value> = msg
                        .tool_calls
                        .iter()
                        .map(|tc| {
                            serde_json::json!({
                                "id": tc.id,
                                "type": "function",
                                "function": {
                                    "name": tc.name,
                                    "arguments": tc.arguments.to_string(),
                                },
                            })
                        })
                        .collect();
                    value["tool_calls"] = serde_json::Value::Array(tool_calls);
                }
                value
            }
            ChatRole::Tool => serde_json::json!({
                "role": "tool",
                "tool_call_id": msg.tool_call_id.as_deref().unwrap_or(""),
                "content": msg.content,
            }),
        })
        .collect()
}

#[derive(Debug, Deserialize)]
struct ChatTurnResponse {
    choices: Vec<ChatTurnChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatTurnChoice {
    message: ChatTurnMessage,
}

#[derive(Debug, Deserialize)]
struct ChatTurnMessage {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<RawToolCall>>,
}

#[derive(Debug, Deserialize)]
struct RawToolCall {
    id: String,
    function: RawToolFunction,
}

#[derive(Debug, Deserialize)]
struct RawToolFunction {
    name: String,
    arguments: String,
}
