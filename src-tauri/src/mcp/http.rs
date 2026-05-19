use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Client;
use tokio::sync::{mpsc, Mutex};

use super::client::{FrameReceiver, Transport};
use super::error::{McpError, McpResult};

pub struct HttpConfig {
    pub url: String,
    pub headers: HashMap<String, String>,
}

pub struct HttpTransport {
    url: String,
    headers: HashMap<String, String>,
    http: Client,
    inbound: Arc<Mutex<Option<mpsc::UnboundedSender<McpResult<String>>>>>,
}

impl HttpTransport {
    pub fn connect(config: HttpConfig) -> McpResult<(Arc<Self>, FrameReceiver)> {
        let http = Client::builder()
            .user_agent("acorn-mcp/0.1")
            .build()
            .map_err(McpError::from)?;

        let (tx, rx) = mpsc::unbounded_channel();
        let transport = Arc::new(Self {
            url: config.url,
            headers: config.headers,
            http,
            inbound: Arc::new(Mutex::new(Some(tx))),
        });
        Ok((transport, rx))
    }
}

#[async_trait]
impl Transport for HttpTransport {
    async fn send(&self, frame: String) -> McpResult<()> {
        let mut builder = self.http.post(&self.url).body(frame);
        builder = builder.header("content-type", "application/json");
        builder = builder.header("accept", "application/json, text/event-stream");
        for (k, v) in &self.headers {
            builder = builder.header(k, v);
        }

        let response = builder.send().await?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(McpError::Transport(format!(
                "HTTP {status}: {}",
                body.chars().take(500).collect::<String>()
            )));
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let text = response.text().await.map_err(McpError::from)?;
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Ok(());
        }

        if content_type.contains("text/event-stream") {
            for frame in parse_sse_payload(trimmed) {
                self.deliver(frame).await?;
            }
        } else {
            self.deliver(trimmed.to_string()).await?;
        }
        Ok(())
    }

    async fn shutdown(&self) -> McpResult<()> {
        let mut guard = self.inbound.lock().await;
        let _ = guard.take();
        Ok(())
    }
}

impl HttpTransport {
    async fn deliver(&self, frame: String) -> McpResult<()> {
        let guard = self.inbound.lock().await;
        let Some(sender) = guard.as_ref() else {
            return Err(McpError::Closed);
        };
        sender.send(Ok(frame)).map_err(|_| McpError::Closed)?;
        Ok(())
    }
}

fn parse_sse_payload(text: &str) -> Vec<String> {
    let mut frames = Vec::new();
    let mut current = String::new();
    for line in text.lines() {
        if line.is_empty() {
            if !current.is_empty() {
                frames.push(std::mem::take(&mut current));
            }
            continue;
        }
        let Some(rest) = line.strip_prefix("data:") else {
            continue;
        };
        let rest = rest.strip_prefix(' ').unwrap_or(rest);
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(rest);
    }
    if !current.is_empty() {
        frames.push(current);
    }
    frames
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sse_payload_single_event() {
        let frames = parse_sse_payload("data: {\"id\":1}");
        assert_eq!(frames, vec!["{\"id\":1}"]);
    }

    #[test]
    fn sse_payload_multiple_events() {
        let raw = "data: {\"id\":1}\n\ndata: {\"id\":2}\n\n";
        let frames = parse_sse_payload(raw);
        assert_eq!(frames, vec!["{\"id\":1}", "{\"id\":2}"]);
    }

    #[test]
    fn sse_payload_multiline_event() {
        let raw = "data: line one\ndata: line two\n\n";
        let frames = parse_sse_payload(raw);
        assert_eq!(frames, vec!["line one\nline two"]);
    }
}
