//! A thin authorized HTTP client shared by the Google Calendar and Tasks
//! providers. Mints an access token from the refresh token on construction and
//! maps Google's status codes onto the engine's control-flow errors.

use super::google_oauth;
use crate::sync::error::{SyncError, SyncResult};

pub struct GoogleClient {
    http: reqwest::Client,
    access_token: String,
}

impl GoogleClient {
    pub async fn connect(refresh_token: &str) -> SyncResult<Self> {
        let access_token = google_oauth::mint_access_token(refresh_token).await?;
        Ok(Self {
            http: reqwest::Client::new(),
            access_token,
        })
    }

    pub fn get(&self, url: &str) -> reqwest::RequestBuilder {
        self.http.get(url)
    }

    pub fn post(&self, url: &str) -> reqwest::RequestBuilder {
        self.http.post(url)
    }

    pub fn patch(&self, url: &str) -> reqwest::RequestBuilder {
        self.http.patch(url)
    }

    pub fn delete(&self, url: &str) -> reqwest::RequestBuilder {
        self.http.delete(url)
    }

    /// Send an authorized request, translating status codes: 401 → Unauthorized,
    /// 410 → FullResyncRequired, 409/412 → Conflict, other 4xx/5xx → Remote.
    /// 404 on delete is treated as success by callers, so it surfaces as Remote
    /// here and is handled at the call site.
    pub async fn send(&self, req: reqwest::RequestBuilder) -> SyncResult<reqwest::Response> {
        let resp = req.bearer_auth(&self.access_token).send().await?;
        let status = resp.status();
        if status.is_success() {
            return Ok(resp);
        }
        match status.as_u16() {
            401 => Err(SyncError::Unauthorized(
                "Google rejected the access token".into(),
            )),
            404 => Err(SyncError::Remote("404 not found".into())),
            410 => Err(SyncError::FullResyncRequired),
            409 | 412 => Err(SyncError::Conflict),
            _ => {
                let body = resp.text().await.unwrap_or_default();
                Err(SyncError::Remote(format!("{status}: {body}")))
            }
        }
    }
}
