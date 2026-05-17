use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::ai::error::{ProviderError, ProviderResult};
use crate::ai::metadata::ProviderMetadata;
use crate::ai::prompts::DECOMPOSE_SYSTEM_PROMPT;
use crate::ai::provider::{emit_decomposed, format_user_input, strip_code_fences, Provider};
use crate::ai::types::{DecomposeEvent, DecomposeRequest, DecomposeResponse};

pub struct OllamaProvider {
    metadata: ProviderMetadata,
    model: String,
    endpoint: String,
    client: Client,
}

impl OllamaProvider {
    pub fn new(metadata: ProviderMetadata, model: String, custom_endpoint: Option<String>) -> Self {
        let endpoint = custom_endpoint.unwrap_or_else(|| metadata.default_endpoint.to_string());
        Self {
            metadata,
            model,
            endpoint,
            client: Client::new(),
        }
    }

    fn chat_url(&self) -> String {
        format!("{}/api/chat", self.endpoint.trim_end_matches('/'))
    }

    fn tags_url(&self) -> String {
        format!("{}/api/tags", self.endpoint.trim_end_matches('/'))
    }
}

#[derive(Debug, Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    messages: Vec<OllamaMessage<'a>>,
    stream: bool,
    format: &'a str,
    options: OllamaOptions,
}

#[derive(Debug, Serialize)]
struct OllamaMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    message: OllamaResponseMessage,
}

#[derive(Debug, Deserialize)]
struct OllamaResponseMessage {
    content: String,
}

#[async_trait]
impl Provider for OllamaProvider {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn decompose(
        &self,
        request: DecomposeRequest,
        events: mpsc::Sender<DecomposeEvent>,
    ) -> ProviderResult<DecomposeResponse> {
        let user_content = format_user_input(&request);

        let body = OllamaRequest {
            model: &self.model,
            messages: vec![
                OllamaMessage {
                    role: "system",
                    content: DECOMPOSE_SYSTEM_PROMPT,
                },
                OllamaMessage {
                    role: "user",
                    content: &user_content,
                },
            ],
            stream: false,
            format: "json",
            options: OllamaOptions { temperature: 0.3 },
        };

        let _ = events
            .send(DecomposeEvent::Progress { received_chars: 0 })
            .await;

        let resp = self
            .client
            .post(self.chat_url())
            .json(&body)
            .send()
            .await
            .map_err(|err| {
                if err.is_connect() {
                    ProviderError::OllamaNotRunning(self.endpoint.clone())
                } else {
                    ProviderError::Network(err.to_string())
                }
            })?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ProviderError::ProviderResponse(text));
        }

        let parsed: OllamaResponse = resp.json().await?;
        let content = parsed.message.content;

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
        let resp = self
            .client
            .get(self.tags_url())
            .send()
            .await
            .map_err(|err| {
                if err.is_connect() {
                    ProviderError::OllamaNotRunning(self.endpoint.clone())
                } else {
                    ProviderError::Network(err.to_string())
                }
            })?;

        if !resp.status().is_success() {
            return Err(ProviderError::ProviderResponse(format!(
                "ollama returned {}",
                resp.status()
            )));
        }
        Ok(())
    }
}
