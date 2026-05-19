use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("transport error: {0}")]
    Transport(String),

    #[error("io error: {0}")]
    Io(String),

    #[error("protocol error: {0}")]
    Protocol(String),

    #[error("server error {code}: {message}")]
    Server { code: i32, message: String },

    #[error("client not initialized")]
    NotInitialized,

    #[error("tool error: {0}")]
    Tool(String),

    #[error("timed out after {0}ms")]
    Timeout(u64),

    #[error("transport closed")]
    Closed,

    #[error("unsupported feature: {0}")]
    Unsupported(String),
}

impl Serialize for McpError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

impl From<std::io::Error> for McpError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl From<serde_json::Error> for McpError {
    fn from(value: serde_json::Error) -> Self {
        Self::Protocol(value.to_string())
    }
}

impl From<reqwest::Error> for McpError {
    fn from(value: reqwest::Error) -> Self {
        Self::Transport(value.to_string())
    }
}

impl From<tokio::time::error::Elapsed> for McpError {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        Self::Timeout(0)
    }
}

pub type McpResult<T> = Result<T, McpError>;
