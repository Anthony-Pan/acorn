use serde::{Deserialize, Serialize};

/// A single transcription request from the frontend.
///
/// `audio` carries the bytes captured by the browser `MediaRecorder` (typically
/// `audio/webm;codecs=opus`). Concrete providers are responsible for either
/// using the bytes directly (Whisper) or decoding them to a format that the
/// native API understands (system providers — see `speech::decoder`).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscribeRequest {
    pub audio: Vec<u8>,
    pub mime_type: String,
    /// Optional BCP-47 language tag (e.g. `"en-US"`, `"zh-CN"`). Hint only — providers may ignore.
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscribeResponse {
    pub text: String,
    /// Id of the provider that actually produced the transcript. Useful when
    /// the system provider falls back to a different backend.
    pub provider_id: String,
}
