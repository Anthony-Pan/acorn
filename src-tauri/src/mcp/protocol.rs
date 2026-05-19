//! JSON-RPC 2.0 envelopes and the MCP subset acorn consumes: initialize,
//! tools/list, tools/call. Resources, prompts, sampling, and notification
//! subscriptions are out of scope for v1.

use serde::{Deserialize, Serialize};

/// `2024-11-05` is the first stable revision all major MCP servers ship
/// (Anthropic / GitHub / Notion / Linear).
pub const PROTOCOL_VERSION: &str = "2024-11-05";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest<P> {
    pub jsonrpc: JsonRpcVersion,
    pub id: RequestId,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<P>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse<R> {
    pub jsonrpc: JsonRpcVersion,
    pub id: RequestId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<R>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification<P> {
    pub jsonrpc: JsonRpcVersion,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<P>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Newtype that only serialises / parses as the literal string `"2.0"`. Used
/// instead of `String` so malformed envelopes are rejected at deserialize time.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "String", into = "String")]
pub struct JsonRpcVersion;

impl TryFrom<String> for JsonRpcVersion {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value == "2.0" {
            Ok(Self)
        } else {
            Err(format!("expected jsonrpc=\"2.0\", got {value:?}"))
        }
    }
}

impl From<JsonRpcVersion> for String {
    fn from(_: JsonRpcVersion) -> Self {
        "2.0".to_string()
    }
}

/// Per JSON-RPC 2.0, ids may be numbers, strings, or null. Real MCP servers in
/// the wild send both numbers (Anthropic, npx servers) and strings (some
/// community servers), so we accept either.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum RequestId {
    Num(u64),
    Str(String),
}

impl From<u64> for RequestId {
    fn from(value: u64) -> Self {
        Self::Num(value)
    }
}

impl From<String> for RequestId {
    fn from(value: String) -> Self {
        Self::Str(value)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    pub client_info: ClientInfo,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ClientCapabilities {}

#[derive(Debug, Clone, Serialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: String,
    #[serde(default)]
    pub capabilities: ServerCapabilities,
    pub server_info: ServerInfo,
    #[serde(default)]
    pub instructions: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ServerCapabilities {
    #[serde(default)]
    pub tools: Option<ToolsCapability>,
    #[serde(flatten)]
    pub _other: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCapability {
    #[serde(default)]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListToolsResult {
    pub tools: Vec<McpTool>,
    #[serde(default, rename = "nextCursor")]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct CallToolParams {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CallToolResult {
    #[serde(default)]
    pub content: Vec<ToolContent>,
    #[serde(default, rename = "isError")]
    pub is_error: bool,
}

/// MCP allows tools to return image, audio, or resource content. The acorn
/// chat layer only forwards text to LLM providers, so non-text variants are
/// surfaced as short JSON placeholders via `to_text`. When we eventually
/// support multimodal tool results, this is the place to extend.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolContent {
    Text {
        text: String,
    },
    Image {
        data: String,
        #[serde(default, rename = "mimeType")]
        mime_type: Option<String>,
    },
    Resource {
        resource: serde_json::Value,
    },
    #[serde(other)]
    Unknown,
}

impl ToolContent {
    pub fn to_text(&self) -> String {
        match self {
            Self::Text { text } => text.clone(),
            Self::Image { mime_type, .. } => {
                let mime = mime_type.as_deref().unwrap_or("image/*");
                format!("[image content omitted; mime={mime}]")
            }
            Self::Resource { resource } => {
                serde_json::to_string(resource).unwrap_or_else(|_| "[resource]".to_string())
            }
            Self::Unknown => "[unknown content]".to_string(),
        }
    }
}

impl CallToolResult {
    pub fn to_text(&self) -> String {
        if self.content.is_empty() {
            return String::new();
        }
        self.content
            .iter()
            .map(ToolContent::to_text)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jsonrpc_version_only_accepts_2_0() {
        let ok = serde_json::from_str::<JsonRpcVersion>(r#""2.0""#);
        assert!(ok.is_ok());

        let bad = serde_json::from_str::<JsonRpcVersion>(r#""1.0""#);
        assert!(bad.is_err());
    }

    #[test]
    fn request_id_is_either_num_or_string() {
        let n: RequestId = serde_json::from_str("42").unwrap();
        assert_eq!(n, RequestId::Num(42));

        let s: RequestId = serde_json::from_str(r#""abc""#).unwrap();
        assert_eq!(s, RequestId::Str("abc".into()));
    }

    #[test]
    fn list_tools_result_parses_real_payload() {
        let raw = r#"{
            "tools": [
                {
                    "name": "echo",
                    "description": "Echoes input back",
                    "inputSchema": {
                        "type": "object",
                        "properties": { "text": { "type": "string" } }
                    }
                }
            ]
        }"#;
        let parsed: ListToolsResult = serde_json::from_str(raw).unwrap();
        assert_eq!(parsed.tools.len(), 1);
        assert_eq!(parsed.tools[0].name, "echo");
        assert_eq!(parsed.tools[0].description.as_deref(), Some("Echoes input back"));
    }

    #[test]
    fn call_tool_result_flattens_text() {
        let raw = r#"{
            "content": [
                { "type": "text", "text": "hello" },
                { "type": "text", "text": "world" }
            ],
            "isError": false
        }"#;
        let parsed: CallToolResult = serde_json::from_str(raw).unwrap();
        assert_eq!(parsed.to_text(), "hello\nworld");
        assert!(!parsed.is_error);
    }

    #[test]
    fn call_tool_result_handles_image_placeholder() {
        let raw = r#"{
            "content": [
                { "type": "text", "text": "screenshot:" },
                { "type": "image", "data": "base64...", "mimeType": "image/png" }
            ]
        }"#;
        let parsed: CallToolResult = serde_json::from_str(raw).unwrap();
        assert!(parsed.to_text().contains("image/png"));
        assert!(!parsed.is_error);
    }

    #[test]
    fn unknown_content_type_does_not_crash() {
        let raw = r#"{"content":[{"type":"audio","data":"..."}]}"#;
        let parsed: CallToolResult = serde_json::from_str(raw).unwrap();
        assert_eq!(parsed.to_text(), "[unknown content]");
    }
}
