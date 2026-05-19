use tauri::State;

use crate::error::AppResult;
use crate::mcp::manager::{McpManager, McpServerConfig};
use crate::mcp::protocol::{CallToolResult, McpTool};

#[tauri::command(rename_all = "camelCase")]
pub async fn list_mcp_servers(
    mcp: State<'_, McpManager>,
) -> AppResult<Vec<McpServerConfig>> {
    Ok(mcp.list_configs().await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_mcp_server(
    id: String,
    mcp: State<'_, McpManager>,
) -> AppResult<Option<McpServerConfig>> {
    Ok(mcp.get_config(&id).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn save_mcp_server(
    config: McpServerConfig,
    mcp: State<'_, McpManager>,
) -> AppResult<McpServerConfig> {
    Ok(mcp.upsert_config(config).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_mcp_server(
    id: String,
    mcp: State<'_, McpManager>,
) -> AppResult<()> {
    Ok(mcp.delete_config(&id).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn test_mcp_server(
    id: String,
    mcp: State<'_, McpManager>,
) -> AppResult<usize> {
    Ok(mcp.test(&id).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn list_mcp_tools(
    server_id: String,
    use_cache: bool,
    mcp: State<'_, McpManager>,
) -> AppResult<Vec<McpTool>> {
    Ok(mcp.list_tools(&server_id, use_cache).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn call_mcp_tool(
    server_id: String,
    name: String,
    arguments: Option<serde_json::Value>,
    mcp: State<'_, McpManager>,
) -> AppResult<CallToolResult> {
    Ok(mcp.call_tool(&server_id, &name, arguments).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn save_mcp_secret(
    server_id: String,
    secret: String,
    mcp: State<'_, McpManager>,
) -> AppResult<()> {
    Ok(mcp.save_secret(&server_id, &secret).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn has_mcp_secret(
    server_id: String,
    mcp: State<'_, McpManager>,
) -> AppResult<bool> {
    Ok(mcp.has_secret(&server_id).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn clear_mcp_secret(
    server_id: String,
    mcp: State<'_, McpManager>,
) -> AppResult<()> {
    Ok(mcp.clear_secret(&server_id).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn start_mcp_server(
    id: String,
    mcp: State<'_, McpManager>,
) -> AppResult<()> {
    Ok(mcp.start(&id).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn stop_mcp_server(
    id: String,
    mcp: State<'_, McpManager>,
) -> AppResult<()> {
    Ok(mcp.stop(&id).await?)
}
