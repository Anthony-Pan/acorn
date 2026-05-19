use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::mpsc;

use super::error::{ProviderError, ProviderResult};
use super::metadata::{ApiFormat, ProviderMetadata};
use super::providers::{
    acorn_cloud::AcornCloudProvider,
    anthropic::AnthropicProvider,
    cli::{invocation_for, CliProvider},
    ollama::OllamaProvider,
    openai_compatible::OpenAiCompatibleProvider,
};
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
