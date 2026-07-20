use chrono::Utc;
use serde::Serialize;
use tauri::State;

use crate::ai::keychain;
use crate::db::models::SpeechProviderConfig;
use crate::db::Database;
use crate::error::AppResult;
use crate::speech::{
    build_speech_provider, default_speech_provider_id, find_speech_metadata,
    speech_provider_catalog, SpeechError, SpeechProviderInputs, SpeechProviderMetadata,
    SpeechResult, TranscribeResponse,
};

pub const ACTIVE_SPEECH_PROVIDER_KEY: &str = "active_speech_provider";

#[tauri::command(rename_all = "camelCase")]
pub fn list_speech_providers() -> Vec<SpeechProviderMetadata> {
    speech_provider_catalog()
}

#[tauri::command(rename_all = "camelCase")]
pub async fn list_speech_provider_configs(
    db: State<'_, Database>,
) -> AppResult<Vec<SpeechProviderConfig>> {
    sqlx::query_as::<_, SpeechProviderConfig>(
        "SELECT * FROM speech_provider_configs \
         ORDER BY last_used_at DESC NULLS LAST, provider_id ASC",
    )
    .fetch_all(db.pool())
    .await
    .map_err(Into::into)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn save_speech_provider_config(
    db: State<'_, Database>,
    provider_id: String,
    enabled: bool,
    selected_language: Option<String>,
) -> AppResult<SpeechProviderConfig> {
    sqlx::query(
        "INSERT INTO speech_provider_configs (provider_id, enabled, selected_language) \
         VALUES (?, ?, ?) \
         ON CONFLICT(provider_id) DO UPDATE SET \
             enabled = excluded.enabled, \
             selected_language = excluded.selected_language",
    )
    .bind(&provider_id)
    .bind(enabled)
    .bind(&selected_language)
    .execute(db.pool())
    .await?;

    sqlx::query_as::<_, SpeechProviderConfig>(
        "SELECT * FROM speech_provider_configs WHERE provider_id = ?",
    )
    .bind(&provider_id)
    .fetch_one(db.pool())
    .await
    .map_err(Into::into)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_active_speech_provider(db: State<'_, Database>) -> AppResult<Option<String>> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
        .bind(ACTIVE_SPEECH_PROVIDER_KEY)
        .fetch_optional(db.pool())
        .await?;
    Ok(row.map(|(v,)| v))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn set_active_speech_provider(
    db: State<'_, Database>,
    provider_id: String,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES (?, ?) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(ACTIVE_SPEECH_PROVIDER_KEY)
    .bind(&provider_id)
    .execute(db.pool())
    .await?;
    Ok(())
}

/// Resolved tuple returned to the frontend so it can show which backend
/// actually produced the transcript (useful when the user is mid-switch
/// or the default is being used).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscribeOutcome {
    pub text: String,
    pub provider_id: String,
}

impl From<TranscribeResponse> for TranscribeOutcome {
    fn from(value: TranscribeResponse) -> Self {
        Self {
            text: value.text,
            provider_id: value.provider_id,
        }
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn transcribe_audio(
    db: State<'_, Database>,
    audio: Vec<u8>,
    mime_type: String,
    language: Option<String>,
) -> SpeechResult<TranscribeOutcome> {
    if audio.is_empty() {
        return Err(SpeechError::UnsupportedFormat(
            "recording was empty — no audio captured".into(),
        ));
    }

    let active = read_active_provider_id(&db)
        .await
        .unwrap_or_else(|| default_speech_provider_id().to_string());

    let metadata = find_speech_metadata(&active)
        .ok_or_else(|| SpeechError::UnknownProvider(active.clone()))?;

    let api_key = match metadata.api_key_provider_id {
        Some(key_id) => keychain::load_api_key(key_id).map_err(map_provider_error)?,
        None => None,
    };

    let provider = build_speech_provider(SpeechProviderInputs { metadata, api_key })?;
    let response = provider
        .transcribe(audio, &mime_type, language.as_deref())
        .await?;

    let _ = sqlx::query(
        "INSERT INTO speech_provider_configs (provider_id, enabled, last_used_at) \
         VALUES (?, 1, ?) \
         ON CONFLICT(provider_id) DO UPDATE SET last_used_at = excluded.last_used_at",
    )
    .bind(&active)
    .bind(Utc::now())
    .execute(db.pool())
    .await;

    Ok(response.into())
}

async fn read_active_provider_id(db: &Database) -> Option<String> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
        .bind(ACTIVE_SPEECH_PROVIDER_KEY)
        .fetch_optional(db.pool())
        .await
        .ok()?;
    row.map(|(v,)| v)
}

fn map_provider_error(err: crate::ai::error::ProviderError) -> SpeechError {
    use crate::ai::error::ProviderError;
    match err {
        ProviderError::InvalidKey => SpeechError::InvalidKey,
        ProviderError::RateLimited => SpeechError::RateLimited,
        ProviderError::Network(s) => SpeechError::Network(s),
        ProviderError::Keychain(s) => SpeechError::Keychain(s),
        ProviderError::MissingCredentials(s) => SpeechError::MissingCredentials(s),
        ProviderError::InvalidResponse(s) => SpeechError::InvalidResponse(s),
        ProviderError::ProviderResponse(s) => SpeechError::ProviderResponse(s),
        ProviderError::NotImplemented(s) => SpeechError::NotImplemented(s),
        ProviderError::OllamaNotRunning(s) => SpeechError::Network(s),
        ProviderError::ChannelClosed => SpeechError::ProviderResponse("channel closed".into()),
        ProviderError::ConversationLocked {
            locked_to,
            attempted,
        } => SpeechError::ProviderResponse(format!(
            "conversation locked to {locked_to}, attempted {attempted}"
        )),
    }
}
