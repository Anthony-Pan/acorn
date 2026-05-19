use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::{oneshot, Mutex};

use super::error::{McpError, McpResult};
use super::protocol::{
    CallToolParams, CallToolResult, ClientCapabilities, ClientInfo, InitializeParams,
    InitializeResult, JsonRpcError, JsonRpcRequest, JsonRpcResponse, JsonRpcVersion,
    ListToolsResult, McpTool, RequestId, PROTOCOL_VERSION,
};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

#[async_trait]
pub trait Transport: Send + Sync {
    async fn send(&self, frame: String) -> McpResult<()>;

    async fn shutdown(&self) -> McpResult<()>;
}

pub type IncomingFrame = String;

pub type FrameReceiver = tokio::sync::mpsc::UnboundedReceiver<McpResult<IncomingFrame>>;

pub struct McpClient {
    transport: Arc<dyn Transport>,
    pending: Arc<Mutex<HashMap<RequestId, oneshot::Sender<JsonRpcResponse<serde_json::Value>>>>>,
    next_id: Arc<Mutex<u64>>,
    timeout: Duration,
    server_info: Arc<Mutex<Option<InitializeResult>>>,
}

impl McpClient {
    pub fn new(transport: Arc<dyn Transport>, frames: FrameReceiver) -> Self {
        let pending = Arc::new(Mutex::new(HashMap::new()));
        let client = Self {
            transport,
            pending: pending.clone(),
            next_id: Arc::new(Mutex::new(1)),
            timeout: DEFAULT_TIMEOUT,
            server_info: Arc::new(Mutex::new(None)),
        };
        client.spawn_reader(frames, pending);
        client
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    fn spawn_reader(
        &self,
        mut frames: FrameReceiver,
        pending: Arc<
            Mutex<HashMap<RequestId, oneshot::Sender<JsonRpcResponse<serde_json::Value>>>>,
        >,
    ) {
        tokio::spawn(async move {
            while let Some(frame) = frames.recv().await {
                let Ok(text) = frame else {
                    continue;
                };
                let parsed: Result<JsonRpcResponse<serde_json::Value>, _> =
                    serde_json::from_str(&text);
                let Ok(response) = parsed else {
                    continue;
                };
                let mut map = pending.lock().await;
                if let Some(sender) = map.remove(&response.id) {
                    let _ = sender.send(response);
                }
            }
        });
    }

    pub async fn initialize(&self, client_name: &str, client_version: &str) -> McpResult<()> {
        let params = InitializeParams {
            protocol_version: PROTOCOL_VERSION.to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: ClientInfo {
                name: client_name.to_string(),
                version: client_version.to_string(),
            },
        };
        let result: InitializeResult = self.request("initialize", Some(params)).await?;
        self.notify("notifications/initialized", None::<serde_json::Value>).await?;
        *self.server_info.lock().await = Some(result);
        Ok(())
    }

    pub async fn server_info(&self) -> Option<InitializeResult> {
        self.server_info.lock().await.clone()
    }

    pub async fn list_tools(&self) -> McpResult<Vec<McpTool>> {
        self.require_initialized().await?;
        let result: ListToolsResult = self.request("tools/list", None::<serde_json::Value>).await?;
        Ok(result.tools)
    }

    pub async fn call_tool(
        &self,
        name: &str,
        arguments: Option<serde_json::Value>,
    ) -> McpResult<CallToolResult> {
        self.require_initialized().await?;
        let params = CallToolParams {
            name: name.to_string(),
            arguments,
        };
        let result: CallToolResult = self.request("tools/call", Some(params)).await?;
        if result.is_error {
            return Err(McpError::Tool(result.to_text()));
        }
        Ok(result)
    }

    pub async fn ping(&self) -> McpResult<()> {
        self.require_initialized().await?;
        let _: serde_json::Value = self.request("ping", None::<serde_json::Value>).await?;
        Ok(())
    }

    pub async fn shutdown(&self) -> McpResult<()> {
        self.transport.shutdown().await
    }

    async fn require_initialized(&self) -> McpResult<()> {
        if self.server_info.lock().await.is_none() {
            return Err(McpError::NotInitialized);
        }
        Ok(())
    }

    async fn request<P, R>(&self, method: &str, params: Option<P>) -> McpResult<R>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        let id = self.allocate_id().await;
        let request = JsonRpcRequest {
            jsonrpc: JsonRpcVersion,
            id: id.clone(),
            method: method.to_string(),
            params,
        };

        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id.clone(), tx);

        let frame = serde_json::to_string(&request)?;
        self.transport.send(frame).await?;

        let response = tokio::time::timeout(self.timeout, rx).await.map_err(|_| {
            McpError::Timeout(self.timeout.as_millis() as u64)
        })?;

        let response = response.map_err(|_| McpError::Closed)?;

        if let Some(err) = response.error {
            return Err(map_jsonrpc_error(err));
        }

        let raw = response
            .result
            .ok_or_else(|| McpError::Protocol("response had neither result nor error".into()))?;
        let parsed = serde_json::from_value::<R>(raw)?;
        Ok(parsed)
    }

    async fn notify<P: Serialize>(&self, method: &str, params: Option<P>) -> McpResult<()> {
        let notif = super::protocol::JsonRpcNotification {
            jsonrpc: JsonRpcVersion,
            method: method.to_string(),
            params,
        };
        let frame = serde_json::to_string(&notif)?;
        self.transport.send(frame).await
    }

    async fn allocate_id(&self) -> RequestId {
        let mut counter = self.next_id.lock().await;
        let id = *counter;
        *counter += 1;
        RequestId::Num(id)
    }
}

fn map_jsonrpc_error(error: JsonRpcError) -> McpError {
    McpError::Server {
        code: error.code,
        message: error.message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    struct FakeTransport {
        outbound: Arc<Mutex<Vec<String>>>,
        inbound: tokio::sync::mpsc::UnboundedSender<McpResult<String>>,
    }

    #[async_trait]
    impl Transport for FakeTransport {
        async fn send(&self, frame: String) -> McpResult<()> {
            let outbound = self.outbound.clone();
            let inbound = self.inbound.clone();
            outbound.lock().await.push(frame.clone());

            let request: JsonRpcRequest<serde_json::Value> = match serde_json::from_str(&frame) {
                Ok(r) => r,
                Err(_) => return Ok(()),
            };

            let response = match request.method.as_str() {
                "initialize" => serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": request.id,
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": { "tools": { "listChanged": true } },
                        "serverInfo": { "name": "fake", "version": "0.0.1" }
                    }
                }),
                "tools/list" => serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": request.id,
                    "result": {
                        "tools": [
                            {
                                "name": "echo",
                                "description": "Echo",
                                "inputSchema": { "type": "object" }
                            }
                        ]
                    }
                }),
                "tools/call" => serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": request.id,
                    "result": {
                        "content": [{ "type": "text", "text": "called" }],
                        "isError": false
                    }
                }),
                "ping" => serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": request.id,
                    "result": {}
                }),
                _ => serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": request.id,
                    "error": { "code": -32601, "message": "method not found" }
                }),
            };
            let _ = inbound.send(Ok(response.to_string()));
            Ok(())
        }

        async fn shutdown(&self) -> McpResult<()> {
            Ok(())
        }
    }

    fn make_client() -> (McpClient, Arc<Mutex<Vec<String>>>) {
        let outbound = Arc::new(Mutex::new(Vec::new()));
        let (inbound_tx, inbound_rx) = mpsc::unbounded_channel();
        let transport = Arc::new(FakeTransport {
            outbound: outbound.clone(),
            inbound: inbound_tx,
        });
        let client = McpClient::new(transport, inbound_rx).with_timeout(Duration::from_secs(2));
        (client, outbound)
    }

    #[tokio::test]
    async fn initialize_handshake_succeeds() {
        let (client, _) = make_client();
        client.initialize("acorn-test", "0.0.1").await.unwrap();
        let info = client.server_info().await.unwrap();
        assert_eq!(info.server_info.name, "fake");
    }

    #[tokio::test]
    async fn list_tools_requires_initialize() {
        let (client, _) = make_client();
        let err = client.list_tools().await.unwrap_err();
        assert!(matches!(err, McpError::NotInitialized));
    }

    #[tokio::test]
    async fn full_roundtrip_returns_tool_payload() {
        let (client, outbound) = make_client();
        client.initialize("acorn-test", "0.0.1").await.unwrap();
        let tools = client.list_tools().await.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "echo");

        let result = client
            .call_tool("echo", Some(serde_json::json!({"text": "hi"})))
            .await
            .unwrap();
        assert_eq!(result.to_text(), "called");

        client.ping().await.unwrap();

        let outbound = outbound.lock().await;
        assert!(outbound.iter().any(|f| f.contains("\"method\":\"initialize\"")));
        assert!(outbound
            .iter()
            .any(|f| f.contains("\"method\":\"notifications/initialized\"")));
        assert!(outbound.iter().any(|f| f.contains("\"method\":\"tools/list\"")));
        assert!(outbound.iter().any(|f| f.contains("\"method\":\"tools/call\"")));
        assert!(outbound.iter().any(|f| f.contains("\"method\":\"ping\"")));
    }

    #[tokio::test]
    async fn server_error_propagates() {
        let (client, _) = make_client();
        client.initialize("acorn-test", "0.0.1").await.unwrap();
        let err = client.call_tool("unknown", None).await.unwrap_err();
        assert!(matches!(err, McpError::Server { .. }));
    }
}
