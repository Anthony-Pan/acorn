use async_trait::async_trait;

use super::error::{SpeechError, SpeechResult};
use super::metadata::{SpeechApiFormat, SpeechProviderMetadata};
use super::types::TranscribeResponse;

#[async_trait]
pub trait SpeechProvider: Send + Sync {
    fn metadata(&self) -> &SpeechProviderMetadata;

    async fn transcribe(
        &self,
        audio: Vec<u8>,
        mime_type: &str,
        language: Option<&str>,
    ) -> SpeechResult<TranscribeResponse>;

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

    match metadata.api_format {
        SpeechApiFormat::System => Err(SpeechError::NotImplemented(
            "system speech provider not yet wired (added in later commit)".into(),
        )),
        SpeechApiFormat::WhisperOpenAi => {
            let _ = api_key;
            Err(SpeechError::NotImplemented(
                "whisper_openai speech provider not yet wired (added in later commit)".into(),
            ))
        }
    }
}
