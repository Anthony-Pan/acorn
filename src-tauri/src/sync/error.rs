use serde::{Serialize, Serializer};

use crate::error::AppError;

/// Errors from the sync subsystem. Mapped into [`AppError::Sync`] at the command
/// boundary; `Conflict` and `FullResyncRequired` are control-flow signals the
/// engine reacts to rather than surfacing verbatim.
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("network error: {0}")]
    Network(String),

    #[error("provider returned error: {0}")]
    Remote(String),

    #[error("authorization required: {0}")]
    Unauthorized(String),

    #[error("oauth flow failed: {0}")]
    OAuth(String),

    #[error("invalid response: {0}")]
    InvalidResponse(String),

    #[error("keychain error: {0}")]
    Keychain(String),

    #[error("missing credentials for {0}")]
    MissingCredentials(String),

    /// Remote changed under us (etag mismatch / HTTP 412) — re-pull and re-resolve.
    #[error("version conflict")]
    Conflict,

    /// Incremental cursor rejected (HTTP 410 GONE) — a full resync is required.
    #[error("sync token expired; full resync required")]
    FullResyncRequired,

    #[error("feature unavailable: {0}")]
    Unavailable(String),

    #[error("not implemented: {0}")]
    NotImplemented(String),

    #[error("database error: {0}")]
    Database(String),
}

impl Serialize for SyncError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

impl From<reqwest::Error> for SyncError {
    fn from(value: reqwest::Error) -> Self {
        if value.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
            return Self::Unauthorized(value.to_string());
        }
        Self::Network(value.to_string())
    }
}

impl From<serde_json::Error> for SyncError {
    fn from(value: serde_json::Error) -> Self {
        Self::InvalidResponse(value.to_string())
    }
}

impl From<keyring::Error> for SyncError {
    fn from(value: keyring::Error) -> Self {
        Self::Keychain(value.to_string())
    }
}

impl From<sqlx::Error> for SyncError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.to_string())
    }
}

impl From<std::io::Error> for SyncError {
    fn from(value: std::io::Error) -> Self {
        Self::Network(value.to_string())
    }
}

impl From<SyncError> for AppError {
    fn from(value: SyncError) -> Self {
        AppError::Sync(value.to_string())
    }
}

pub type SyncResult<T> = Result<T, SyncError>;
