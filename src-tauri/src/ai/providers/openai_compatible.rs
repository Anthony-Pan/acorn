use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::ai::error::{ProviderError, ProviderResult};
use crate::ai::metadata::ProviderMetadata;
use crate::ai::prompts::DECOMPOSE_SYSTEM_PROMPT;
use crate::ai::provider::{emit_decomposed, format_user_input, strip_code_fences, Provider};
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
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
}

#[derive(Debug, Serialize)]
struct ChatMessage<'a> {
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
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChatChoiceMessage {
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

        let body = ChatRequest {
            model: &self.model,
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: DECOMPOSE_SYSTEM_PROMPT,
                },
                ChatMessage {
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

        let parsed: ChatResponse = resp.json().await?;
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

        let json = strip_code_fences(&content);
        let response: DecomposeResponse = serde_json::from_str(json)?;

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
}
