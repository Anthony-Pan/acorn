use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use tokio::sync::Mutex;
use uuid::Uuid;

use super::client::McpClient;
use super::error::{McpError, McpResult};
use super::http::{HttpConfig, HttpTransport};
use super::keychain;
use super::protocol::{CallToolResult, McpTool};
use super::stdio::{StdioConfig, StdioTransport};

const CLIENT_NAME: &str = "acorn";
const CLIENT_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum McpTransport {
    Local,
    Remote,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    pub transport: McpTransport,
    #[serde(default)]
    pub command: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

fn default_timeout_ms() -> u64 {
    30_000
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedTool {
    pub server_id: String,
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

pub struct McpManager {
    pool: SqlitePool,
    clients: Arc<Mutex<HashMap<String, Arc<McpClient>>>>,
}

impl McpManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn list_configs(&self) -> McpResult<Vec<McpServerConfig>> {
        let rows = sqlx::query(
            "SELECT id, name, transport, command_json, env_json, url, headers_json,
                    enabled, timeout_ms, last_error, created_at, updated_at
             FROM mcp_servers
             ORDER BY name ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx)?;

        rows.into_iter().map(row_to_config).collect()
    }

    pub async fn get_config(&self, id: &str) -> McpResult<Option<McpServerConfig>> {
        let row = sqlx::query(
            "SELECT id, name, transport, command_json, env_json, url, headers_json,
                    enabled, timeout_ms, last_error, created_at, updated_at
             FROM mcp_servers
             WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx)?;
        row.map(row_to_config).transpose()
    }

    pub async fn upsert_config(&self, mut config: McpServerConfig) -> McpResult<McpServerConfig> {
        validate_config(&config)?;
        if config.id.is_empty() {
            config.id = Uuid::new_v4().to_string();
        }
        let command_json = if config.transport == McpTransport::Local {
            Some(serde_json::to_string(&config.command)?)
        } else {
            None
        };
        let env_json = if config.transport == McpTransport::Local {
            Some(serde_json::to_string(&config.env)?)
        } else {
            None
        };
        let headers_json = if config.transport == McpTransport::Remote {
            Some(serde_json::to_string(&config.headers)?)
        } else {
            None
        };
        let url = if config.transport == McpTransport::Remote {
            config.url.clone()
        } else {
            None
        };
        let transport_str = match config.transport {
            McpTransport::Local => "local",
            McpTransport::Remote => "remote",
        };
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO mcp_servers
                (id, name, transport, command_json, env_json, url, headers_json,
                 enabled, timeout_ms, last_error, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                transport = excluded.transport,
                command_json = excluded.command_json,
                env_json = excluded.env_json,
                url = excluded.url,
                headers_json = excluded.headers_json,
                enabled = excluded.enabled,
                timeout_ms = excluded.timeout_ms,
                updated_at = excluded.updated_at",
        )
        .bind(&config.id)
        .bind(&config.name)
        .bind(transport_str)
        .bind(&command_json)
        .bind(&env_json)
        .bind(&url)
        .bind(&headers_json)
        .bind(config.enabled as i64)
        .bind(config.timeout_ms as i64)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;

        config.created_at.get_or_insert(now);
        config.updated_at = Some(now);
        Ok(config)
    }

    pub async fn delete_config(&self, id: &str) -> McpResult<()> {
        self.stop(id).await.ok();
        let _ = keychain::delete_secret(id);
        sqlx::query("DELETE FROM mcp_servers WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx)?;
        Ok(())
    }

    pub async fn save_secret(&self, server_id: &str, secret: &str) -> McpResult<()> {
        if self.get_config(server_id).await?.is_none() {
            return Err(McpError::Transport(format!("no server {server_id}")));
        }
        keychain::save_secret(server_id, secret)
    }

    pub async fn has_secret(&self, server_id: &str) -> McpResult<bool> {
        Ok(keychain::load_secret(server_id)?.is_some())
    }

    pub async fn clear_secret(&self, server_id: &str) -> McpResult<()> {
        keychain::delete_secret(server_id)
    }

    pub async fn start(&self, id: &str) -> McpResult<()> {
        let mut config = self
            .get_config(id)
            .await?
            .ok_or_else(|| McpError::Transport(format!("no server {id}")))?;
        let secret = keychain::load_secret(id)?;

        if let Some(s) = secret.as_deref() {
            apply_secret(&mut config.env, s);
            apply_secret(&mut config.headers, s);
        }

        let client = match config.transport {
            McpTransport::Local => {
                let (transport, frames) = StdioTransport::spawn(StdioConfig {
                    command: config.command.clone(),
                    env: config.env.clone(),
                })?;
                McpClient::new(transport, frames)
                    .with_timeout(Duration::from_millis(config.timeout_ms))
            }
            McpTransport::Remote => {
                let url = config
                    .url
                    .clone()
                    .ok_or_else(|| McpError::Transport("remote transport missing url".into()))?;
                let (transport, frames) = HttpTransport::connect(HttpConfig {
                    url,
                    headers: config.headers.clone(),
                })?;
                McpClient::new(transport, frames)
                    .with_timeout(Duration::from_millis(config.timeout_ms))
            }
        };

        client.initialize(CLIENT_NAME, CLIENT_VERSION).await?;

        let client = Arc::new(client);
        self.clients.lock().await.insert(id.to_string(), client);
        self.record_success(id).await;
        Ok(())
    }

    pub async fn stop(&self, id: &str) -> McpResult<()> {
        let removed = self.clients.lock().await.remove(id);
        if let Some(client) = removed {
            client.shutdown().await?;
        }
        Ok(())
    }

    pub async fn is_running(&self, id: &str) -> bool {
        self.clients.lock().await.contains_key(id)
    }

    async fn get_or_start(&self, id: &str) -> McpResult<Arc<McpClient>> {
        if let Some(client) = self.clients.lock().await.get(id).cloned() {
            return Ok(client);
        }
        self.start(id).await?;
        self.clients
            .lock()
            .await
            .get(id)
            .cloned()
            .ok_or(McpError::Closed)
    }

    pub async fn list_tools(&self, id: &str, use_cache: bool) -> McpResult<Vec<McpTool>> {
        if use_cache {
            let cached = self.cached_tools(id).await?;
            if !cached.is_empty() {
                return Ok(cached);
            }
        }
        let client = self.get_or_start(id).await?;
        let tools = match client.list_tools().await {
            Ok(t) => t,
            Err(err) => {
                self.record_error(id, &err.to_string()).await;
                return Err(err);
            }
        };
        self.replace_cache(id, &tools).await?;
        Ok(tools)
    }

    pub async fn call_tool(
        &self,
        id: &str,
        name: &str,
        args: Option<serde_json::Value>,
    ) -> McpResult<CallToolResult> {
        let client = self.get_or_start(id).await?;
        match client.call_tool(name, args).await {
            Ok(result) => Ok(result),
            Err(err) => {
                self.record_error(id, &err.to_string()).await;
                Err(err)
            }
        }
    }

    pub async fn test(&self, id: &str) -> McpResult<usize> {
        self.stop(id).await.ok();
        match self.start(id).await {
            Ok(()) => {
                let tools = self.list_tools(id, false).await?;
                Ok(tools.len())
            }
            Err(err) => {
                self.record_error(id, &err.to_string()).await;
                Err(err)
            }
        }
    }

    pub async fn shutdown_all(&self) -> McpResult<()> {
        let drained: Vec<Arc<McpClient>> =
            self.clients.lock().await.drain().map(|(_, v)| v).collect();
        for client in drained {
            let _ = client.shutdown().await;
        }
        Ok(())
    }

    async fn cached_tools(&self, server_id: &str) -> McpResult<Vec<McpTool>> {
        let rows = sqlx::query(
            "SELECT name, description, input_schema_json
             FROM mcp_tools_cache
             WHERE server_id = ?
             ORDER BY name ASC",
        )
        .bind(server_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx)?;

        rows.into_iter()
            .map(|row| {
                let schema_str: String = row.try_get("input_schema_json").map_err(map_sqlx)?;
                let input_schema = serde_json::from_str(&schema_str)?;
                Ok(McpTool {
                    name: row.try_get("name").map_err(map_sqlx)?,
                    description: row.try_get::<Option<String>, _>("description").map_err(map_sqlx)?,
                    input_schema,
                })
            })
            .collect()
    }

    async fn replace_cache(&self, server_id: &str, tools: &[McpTool]) -> McpResult<()> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx)?;
        sqlx::query("DELETE FROM mcp_tools_cache WHERE server_id = ?")
            .bind(server_id)
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx)?;
        for tool in tools {
            sqlx::query(
                "INSERT INTO mcp_tools_cache
                    (server_id, name, description, input_schema_json, last_seen_at)
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(server_id)
            .bind(&tool.name)
            .bind(&tool.description)
            .bind(serde_json::to_string(&tool.input_schema)?)
            .bind(Utc::now())
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx)?;
        }
        tx.commit().await.map_err(map_sqlx)?;
        Ok(())
    }

    async fn record_success(&self, id: &str) {
        let _ = sqlx::query("UPDATE mcp_servers SET last_error = NULL, updated_at = ? WHERE id = ?")
            .bind(Utc::now())
            .bind(id)
            .execute(&self.pool)
            .await;
    }

    async fn record_error(&self, id: &str, message: &str) {
        let _ = sqlx::query("UPDATE mcp_servers SET last_error = ?, updated_at = ? WHERE id = ?")
            .bind(message)
            .bind(Utc::now())
            .bind(id)
            .execute(&self.pool)
            .await;
    }
}

fn validate_config(config: &McpServerConfig) -> McpResult<()> {
    if config.name.trim().is_empty() {
        return Err(McpError::Transport("name is required".into()));
    }
    match config.transport {
        McpTransport::Local => {
            if config.command.is_empty() {
                return Err(McpError::Transport("local transport needs command".into()));
            }
        }
        McpTransport::Remote => {
            let url_ok = config
                .url
                .as_deref()
                .map(|u| u.starts_with("http://") || u.starts_with("https://"))
                .unwrap_or(false);
            if !url_ok {
                return Err(McpError::Transport("remote transport needs http(s) url".into()));
            }
        }
    }
    Ok(())
}

fn apply_secret(map: &mut HashMap<String, String>, secret: &str) {
    for value in map.values_mut() {
        if value.contains("${SECRET}") {
            *value = value.replace("${SECRET}", secret);
        }
    }
}

fn row_to_config(row: sqlx::sqlite::SqliteRow) -> McpResult<McpServerConfig> {
    let transport_str: String = row.try_get("transport").map_err(map_sqlx)?;
    let transport = match transport_str.as_str() {
        "local" => McpTransport::Local,
        "remote" => McpTransport::Remote,
        other => return Err(McpError::Protocol(format!("unknown transport {other}"))),
    };
    let command_json: Option<String> = row.try_get("command_json").map_err(map_sqlx)?;
    let env_json: Option<String> = row.try_get("env_json").map_err(map_sqlx)?;
    let headers_json: Option<String> = row.try_get("headers_json").map_err(map_sqlx)?;

    let command = command_json
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?
        .unwrap_or_default();
    let env = env_json
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?
        .unwrap_or_default();
    let headers = headers_json
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?
        .unwrap_or_default();

    let enabled_int: i64 = row.try_get("enabled").map_err(map_sqlx)?;
    let timeout_int: i64 = row.try_get("timeout_ms").map_err(map_sqlx)?;

    Ok(McpServerConfig {
        id: row.try_get("id").map_err(map_sqlx)?,
        name: row.try_get("name").map_err(map_sqlx)?,
        transport,
        command,
        env,
        url: row.try_get("url").map_err(map_sqlx)?,
        headers,
        enabled: enabled_int != 0,
        timeout_ms: timeout_int as u64,
        last_error: row.try_get("last_error").map_err(map_sqlx)?,
        created_at: row.try_get("created_at").map_err(map_sqlx)?,
        updated_at: row.try_get("updated_at").map_err(map_sqlx)?,
    })
}

fn map_sqlx(err: sqlx::Error) -> McpError {
    McpError::Transport(format!("db: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    }

    fn local_config(name: &str) -> McpServerConfig {
        McpServerConfig {
            id: String::new(),
            name: name.into(),
            transport: McpTransport::Local,
            command: vec!["cat".into()],
            env: HashMap::new(),
            url: None,
            headers: HashMap::new(),
            enabled: true,
            timeout_ms: 5_000,
            last_error: None,
            created_at: None,
            updated_at: None,
        }
    }

    fn remote_config(name: &str, url: &str) -> McpServerConfig {
        McpServerConfig {
            id: String::new(),
            name: name.into(),
            transport: McpTransport::Remote,
            command: vec![],
            env: HashMap::new(),
            url: Some(url.into()),
            headers: HashMap::new(),
            enabled: true,
            timeout_ms: 5_000,
            last_error: None,
            created_at: None,
            updated_at: None,
        }
    }

    #[tokio::test]
    async fn upsert_and_list_roundtrip() {
        let pool = setup_db().await;
        let mgr = McpManager::new(pool);

        let saved = mgr.upsert_config(local_config("alpha")).await.unwrap();
        assert!(!saved.id.is_empty());
        assert!(saved.created_at.is_some());

        let listed = mgr.list_configs().await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "alpha");
        assert_eq!(listed[0].command, vec!["cat".to_string()]);
        assert_eq!(listed[0].transport, McpTransport::Local);
    }

    #[tokio::test]
    async fn local_requires_command() {
        let pool = setup_db().await;
        let mgr = McpManager::new(pool);
        let mut cfg = local_config("bad");
        cfg.command.clear();
        let err = mgr.upsert_config(cfg).await.unwrap_err();
        assert!(matches!(err, McpError::Transport(_)));
    }

    #[tokio::test]
    async fn remote_requires_http_url() {
        let pool = setup_db().await;
        let mgr = McpManager::new(pool);
        let mut cfg = remote_config("bad", "ftp://example.com");
        let err = mgr.upsert_config(cfg.clone()).await.unwrap_err();
        assert!(matches!(err, McpError::Transport(_)));

        cfg.url = Some("https://example.com/mcp".into());
        let saved = mgr.upsert_config(cfg).await.unwrap();
        assert_eq!(saved.transport, McpTransport::Remote);
    }

    #[tokio::test]
    async fn delete_removes_row() {
        let pool = setup_db().await;
        let mgr = McpManager::new(pool);
        let saved = mgr.upsert_config(local_config("gamma")).await.unwrap();
        mgr.delete_config(&saved.id).await.unwrap();
        let listed = mgr.list_configs().await.unwrap();
        assert!(listed.is_empty());
    }

    #[tokio::test]
    async fn cache_replace_overwrites_old_tools() {
        let pool = setup_db().await;
        let mgr = McpManager::new(pool);
        let saved = mgr.upsert_config(local_config("delta")).await.unwrap();

        let tools = vec![McpTool {
            name: "echo".into(),
            description: Some("Echo a string".into()),
            input_schema: serde_json::json!({"type": "object"}),
        }];
        mgr.replace_cache(&saved.id, &tools).await.unwrap();
        let cached = mgr.cached_tools(&saved.id).await.unwrap();
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].name, "echo");

        let tools_b = vec![McpTool {
            name: "ping".into(),
            description: None,
            input_schema: serde_json::json!({"type": "object"}),
        }];
        mgr.replace_cache(&saved.id, &tools_b).await.unwrap();
        let cached_b = mgr.cached_tools(&saved.id).await.unwrap();
        assert_eq!(cached_b.len(), 1);
        assert_eq!(cached_b[0].name, "ping");
    }

    #[tokio::test]
    async fn apply_secret_replaces_template_in_env() {
        let mut env = HashMap::new();
        env.insert("GITHUB_TOKEN".into(), "${SECRET}".into());
        env.insert("LOG_LEVEL".into(), "info".into());
        apply_secret(&mut env, "ghp_xyz");
        assert_eq!(env.get("GITHUB_TOKEN").unwrap(), "ghp_xyz");
        assert_eq!(env.get("LOG_LEVEL").unwrap(), "info");
    }
}
