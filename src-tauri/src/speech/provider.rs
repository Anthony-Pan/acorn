use async_trait::async_trait;

use super::error::{SpeechError, SpeechResult};
use super::metadata::{SpeechApiFormat, SpeechProviderMetadata, SpeechProviderStatus};
use super::providers::whisper_cloud::WhisperCloudProvider;
use super::types::TranscribeResponse;

#[async_trait]
pub trait SpeechProvider: Send + Sync {
    #[allow(dead_code)]
    fn metadata(&self) -> &SpeechProviderMetadata;

    async fn transcribe(
        &self,
        audio: Vec<u8>,
        mime_type: &str,
        language: Option<&str>,
    ) -> SpeechResult<TranscribeResponse>;

    // Reserved for a future test_speech_provider_connection command (mirrors
    // the existing test_provider_connection for AI providers).
    #[allow(dead_code)]
    async fn validate_availability(&self) -> SpeechResult<()>;
}

pub struct SpeechProviderInputs {
    pub metadata: SpeechProviderMetadata,
    pub api_key: Option<String>,
}

pub fn build_speech_provider(
    inputs: SpeechProviderInputs,
) -> SpeechResult<Box<dyn SpeechProvider>> {
    let SpeechProviderInputs { metadata, api_key } = inputs;

    if !metadata.available_on_current_platform() {
        return Err(SpeechError::Unavailable);
    }
    if metadata.status == SpeechProviderStatus::ComingSoon {
        return Err(SpeechError::NotImplemented(format!(
            "{} ships in v1.1",
            metadata.display_name
        )));
    }

    match metadata.api_format {
        SpeechApiFormat::System => Err(SpeechError::NotImplemented(
            "system speech provider lands in v1.1".into(),
        )),
        SpeechApiFormat::WhisperOpenAi => Ok(Box::new(WhisperCloudProvider::new(
            metadata,
            api_key.unwrap_or_default(),
        ))),
    }
}
