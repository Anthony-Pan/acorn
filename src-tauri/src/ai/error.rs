use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("invalid API key")]
    InvalidKey,

    #[error("rate limit exceeded")]
    RateLimited,

    #[error("network error: {0}")]
    Network(String),

    #[error("invalid response from model: {0}")]
    InvalidResponse(String),

    #[error("provider returned error: {0}")]
    ProviderResponse(String),

    #[error("ollama not running at {0}. start it with 'ollama serve'")]
    OllamaNotRunning(String),

    #[error("not implemented: {0}")]
    NotImplemented(String),

    #[error("keychain error: {0}")]
    Keychain(String),

    #[error("missing credentials for {0}")]
    MissingCredentials(String),

    #[error("channel closed")]
    ChannelClosed,
}

impl Serialize for ProviderError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

impl From<reqwest::Error> for ProviderError {
    fn from(value: reqwest::Error) -> Self {
        if let Some(status) = value.status() {
            if status == reqwest::StatusCode::UNAUTHORIZED {
                return Self::InvalidKey;
            }
            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                return Self::RateLimited;
            }
        }
        Self::Network(value.to_string())
    }
}

impl From<serde_json::Error> for ProviderError {
    fn from(value: serde_json::Error) -> Self {
        Self::InvalidResponse(value.to_string())
    }
}

impl From<keyring::Error> for ProviderError {
    fn from(value: keyring::Error) -> Self {
        Self::Keychain(value.to_string())
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for ProviderError {
    fn from(_: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::ChannelClosed
    }
}

impl From<sqlx::Error> for ProviderError {
    fn from(value: sqlx::Error) -> Self {
        Self::ProviderResponse(format!("database error: {value}"))
    }
}

pub type ProviderResult<T> = Result<T, ProviderError>;
