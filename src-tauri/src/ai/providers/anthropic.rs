use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::ai::error::{ProviderError, ProviderResult};
use crate::ai::metadata::ProviderMetadata;
use crate::ai::prompts::DECOMPOSE_SYSTEM_PROMPT;
use crate::ai::provider::{
    emit_decomposed, format_user_input, strip_code_fences, Provider,
};
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
    Text { text: String },
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
            return Err(ProviderError::ProviderResponse(format!("HTTP {status}: {text}")));
        }

        let parsed: AnthropicResponse = resp.json().await?;
        let text = parsed
            .content
            .into_iter()
            .find_map(|c| match c {
                AnthropicContent::Text { text } => Some(text),
                AnthropicContent::Other => None,
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
}
