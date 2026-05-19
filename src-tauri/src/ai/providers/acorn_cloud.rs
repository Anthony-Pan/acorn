use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::ai::chat::{ChatMessage, ChatTurn};
use crate::ai::error::{ProviderError, ProviderResult};
use crate::ai::metadata::ProviderMetadata;
use crate::ai::provider::Provider;
use crate::ai::tools::ToolSpec;
use crate::ai::types::{DecomposeEvent, DecomposeRequest, DecomposeResponse};

pub struct AcornCloudProvider {
    metadata: ProviderMetadata,
}

impl AcornCloudProvider {
    pub fn new(metadata: ProviderMetadata) -> Self {
        Self { metadata }
    }
}

#[async_trait]
impl Provider for AcornCloudProvider {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn decompose(
        &self,
        _request: DecomposeRequest,
        _events: mpsc::Sender<DecomposeEvent>,
    ) -> ProviderResult<DecomposeResponse> {
        Err(ProviderError::NotImplemented(
            "Acorn Cloud is coming soon. Join the waitlist at acorn.app.".into(),
        ))
    }

    async fn validate_credentials(&self) -> ProviderResult<()> {
        Err(ProviderError::NotImplemented(
            "Acorn Cloud is not yet available.".into(),
        ))
    }

    async fn chat_turn(
        &self,
        _messages: &[ChatMessage],
        _tools: &[ToolSpec],
        _system_prompt: &str,
    ) -> ProviderResult<ChatTurn> {
        Err(ProviderError::NotImplemented(
            "Acorn Cloud is coming soon. Join the waitlist at acorn.app.".into(),
        ))
    }
}
