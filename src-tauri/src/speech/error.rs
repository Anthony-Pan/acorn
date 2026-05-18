use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum SpeechError {
    #[error("speech recognition is not available on this platform")]
    Unavailable,

    #[error("permission to use speech recognition was denied")]
    PermissionDenied,

    #[error("missing credentials for {0}")]
    MissingCredentials(String),

    #[error("invalid API key")]
    InvalidKey,

    #[error("rate limit exceeded")]
    RateLimited,

    #[error("unsupported audio format: {0}")]
    UnsupportedFormat(String),

    #[error("audio decode error: {0}")]
    Decode(String),

    #[error("native speech API error: {0}")]
    NativeApi(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("provider returned error: {0}")]
    ProviderResponse(String),

    #[error("invalid response: {0}")]
    InvalidResponse(String),

    #[error("io error: {0}")]
    Io(String),

    #[error("keychain error: {0}")]
    Keychain(String),

    #[error("unknown speech provider: {0}")]
    UnknownProvider(String),

    #[error("not implemented: {0}")]
    NotImplemented(String),
}

impl Serialize for SpeechError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

impl From<reqwest::Error> for SpeechError {
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

impl From<serde_json::Error> for SpeechError {
    fn from(value: serde_json::Error) -> Self {
        Self::InvalidResponse(value.to_string())
    }
}

impl From<keyring::Error> for SpeechError {
    fn from(value: keyring::Error) -> Self {
        Self::Keychain(value.to_string())
    }
}

impl From<std::io::Error> for SpeechError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl From<sqlx::Error> for SpeechError {
    fn from(value: sqlx::Error) -> Self {
        Self::ProviderResponse(format!("database error: {value}"))
    }
}

pub type SpeechResult<T> = Result<T, SpeechError>;
