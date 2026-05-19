use chrono::Utc;
use tauri::ipc::Channel;
use tauri::State;
use tokio::sync::mpsc;

use crate::ai::error::ProviderResult;
use crate::ai::metadata::find_metadata;
use crate::ai::provider::{build_provider, ProviderInputs};
use crate::ai::{
    keychain, provider_catalog, DecomposeEvent, DecomposeRequest, DecomposeResponse, ProviderError,
    ProviderMetadata,
};
use crate::db::models::ProviderConfig;
use crate::db::Database;

#[tauri::command(rename_all = "camelCase")]
pub fn list_providers() -> Vec<ProviderMetadata> {
    provider_catalog()
}

#[tauri::command(rename_all = "camelCase")]
pub async fn list_ollama_models(custom_endpoint: Option<String>) -> ProviderResult<Vec<String>> {
    #[derive(serde::Deserialize)]
    struct TagsResponse {
        models: Vec<TagEntry>,
    }
    #[derive(serde::Deserialize)]
    struct TagEntry {
        name: String,
    }

    let endpoint = custom_endpoint
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "http://localhost:11434".to_string());
    let url = format!("{}/api/tags", endpoint.trim_end_matches('/'));

    let resp = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .map_err(|err| {
            if err.is_connect() {
                ProviderError::OllamaNotRunning(endpoint.clone())
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

    let parsed: TagsResponse = resp.json().await?;
    Ok(parsed.models.into_iter().map(|m| m.name).collect())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn save_provider_credentials(provider_id: String, api_key: String) -> ProviderResult<()> {
    keychain::save_api_key(&provider_id, &api_key)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_provider_credentials(provider_id: String) -> ProviderResult<()> {
    keychain::delete_api_key(&provider_id)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn has_provider_credentials(provider_id: String) -> ProviderResult<bool> {
    Ok(keychain::load_api_key(&provider_id)?.is_some())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn test_provider_connection(
    db: State<'_, Database>,
    provider_id: String,
) -> ProviderResult<()> {
    let provider = resolve_provider(&db, &provider_id).await?;
    provider.validate_credentials().await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn decompose(
    db: State<'_, Database>,
    provider_id: String,
    request: DecomposeRequest,
    on_event: Channel<DecomposeEvent>,
) -> ProviderResult<DecomposeResponse> {
    let provider = resolve_provider(&db, &provider_id).await?;
    let (tx, mut rx) = mpsc::channel::<DecomposeEvent>(32);

    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            let _ = on_event.send(event);
        }
    });

    let response = provider.decompose(request, tx).await?;

    let _ = sqlx::query("UPDATE provider_configs SET last_used_at = ? WHERE provider_id = ?")
        .bind(Utc::now())
        .bind(provider.metadata().id)
        .execute(db.pool())
        .await;

    Ok(response)
}

async fn resolve_provider(
    db: &Database,
    provider_id: &str,
) -> ProviderResult<Box<dyn crate::ai::Provider>> {
    let metadata = find_metadata(provider_id)
        .ok_or_else(|| ProviderError::NotImplemented(format!("unknown provider {provider_id}")))?;

    let api_key = keychain::load_api_key(provider_id)?;
    let config =
        sqlx::query_as::<_, ProviderConfig>("SELECT * FROM provider_configs WHERE provider_id = ?")
            .bind(provider_id)
            .fetch_optional(db.pool())
            .await?;

    build_provider(ProviderInputs {
        metadata,
        api_key,
        model: config.as_ref().and_then(|c| c.selected_model.clone()),
        custom_endpoint: config.and_then(|c| c.custom_endpoint),
    })
}
