use async_trait::async_trait;

use crate::ai::error::ProviderError;
use crate::ai::transcribe;
use crate::speech::error::{SpeechError, SpeechResult};
use crate::speech::metadata::SpeechProviderMetadata;
use crate::speech::provider::SpeechProvider;
use crate::speech::types::TranscribeResponse;

pub struct WhisperCloudProvider {
    metadata: SpeechProviderMetadata,
    api_key: String,
}

impl WhisperCloudProvider {
    pub fn new(metadata: SpeechProviderMetadata, api_key: String) -> Self {
        Self { metadata, api_key }
    }

    fn require_key(&self) -> SpeechResult<&str> {
        if self.api_key.is_empty() {
            return Err(SpeechError::MissingCredentials(
                self.metadata
                    .api_key_provider_id
                    .unwrap_or(self.metadata.id)
                    .to_string(),
            ));
        }
        Ok(&self.api_key)
    }
}

#[async_trait]
impl SpeechProvider for WhisperCloudProvider {
    fn metadata(&self) -> &SpeechProviderMetadata {
        &self.metadata
    }

    async fn transcribe(
        &self,
        audio: Vec<u8>,
        mime_type: &str,
        language: Option<&str>,
    ) -> SpeechResult<TranscribeResponse> {
        let api_key = self.require_key()?;
        // Whisper expects ISO 639-1 ("en"); frontend sends BCP-47 ("en-US"). Strip the region.
        let whisper_lang = language.and_then(|l| l.split('-').next());
        let text = transcribe::transcribe_audio(api_key, audio, mime_type, whisper_lang)
            .await
            .map_err(provider_error_to_speech)?;
        Ok(TranscribeResponse {
            text,
            provider_id: self.metadata.id.to_string(),
        })
    }

    async fn validate_availability(&self) -> SpeechResult<()> {
        self.require_key()?;
        Ok(())
    }
}

fn provider_error_to_speech(err: ProviderError) -> SpeechError {
    match err {
        ProviderError::InvalidKey => SpeechError::InvalidKey,
        ProviderError::RateLimited => SpeechError::RateLimited,
        ProviderError::Network(s) => SpeechError::Network(s),
        ProviderError::InvalidResponse(s) => SpeechError::InvalidResponse(s),
        ProviderError::ProviderResponse(s) => SpeechError::ProviderResponse(s),
        ProviderError::MissingCredentials(s) => SpeechError::MissingCredentials(s),
        ProviderError::Keychain(s) => SpeechError::Keychain(s),
        ProviderError::NotImplemented(s) => SpeechError::NotImplemented(s),
        ProviderError::OllamaNotRunning(s) => SpeechError::Network(s),
        ProviderError::ChannelClosed => SpeechError::ProviderResponse("channel closed".into()),
    }
}
