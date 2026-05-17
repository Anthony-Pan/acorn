use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde::Deserialize;

use super::error::{ProviderError, ProviderResult};

const ENDPOINT: &str = "https://api.openai.com/v1/audio/transcriptions";
const MODEL: &str = "whisper-1";

#[derive(Debug, Deserialize)]
struct TranscriptionResponse {
    text: String,
}

pub async fn transcribe_audio(
    api_key: &str,
    audio: Vec<u8>,
    mime_type: &str,
    language: Option<&str>,
) -> ProviderResult<String> {
    if api_key.is_empty() {
        return Err(ProviderError::MissingCredentials("openai".into()));
    }

    let filename = filename_for(mime_type);
    let part = Part::bytes(audio)
        .file_name(filename)
        .mime_str(mime_type)
        .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;

    let mut form = Form::new().part("file", part).text("model", MODEL);
    if let Some(lang) = language {
        form = form.text("language", lang.to_string());
    }

    let resp = Client::new()
        .post(ENDPOINT)
        .bearer_auth(api_key)
        .multipart(form)
        .send()
        .await?;

    if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(ProviderError::InvalidKey);
    }
    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(ProviderError::ProviderResponse(text));
    }

    let parsed: TranscriptionResponse = resp.json().await?;
    Ok(parsed.text)
}

fn filename_for(mime: &str) -> &'static str {
    if mime.contains("webm") {
        "audio.webm"
    } else if mime.contains("mp4") || mime.contains("m4a") {
        "audio.m4a"
    } else if mime.contains("ogg") {
        "audio.ogg"
    } else if mime.contains("wav") {
        "audio.wav"
    } else if mime.contains("mpeg") {
        "audio.mp3"
    } else {
        "audio.bin"
    }
}
