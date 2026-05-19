use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{mpsc, Mutex};

use super::client::{FrameReceiver, Transport};
use super::error::{McpError, McpResult};

pub struct StdioConfig {
    pub command: Vec<String>,
    pub env: HashMap<String, String>,
}

pub struct StdioTransport {
    child: Arc<Mutex<Option<Child>>>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
}

impl StdioTransport {
    pub fn spawn(config: StdioConfig) -> McpResult<(Arc<Self>, FrameReceiver)> {
        let mut args = config.command.into_iter();
        let program = args
            .next()
            .ok_or_else(|| McpError::Transport("empty command".into()))?;
        let argv: Vec<String> = args.collect();

        let mut cmd = Command::new(&program);
        cmd.args(&argv)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        for (k, v) in config.env {
            cmd.env(k, v);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| McpError::Transport(format!("failed to spawn {program}: {e}")))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| McpError::Transport("stdin pipe unavailable".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| McpError::Transport("stdout pipe unavailable".into()))?;
        let stderr = child.stderr.take();

        let (frame_tx, frame_rx) = mpsc::unbounded_channel();
        spawn_stdout_reader(stdout, frame_tx.clone());
        if let Some(err) = stderr {
            spawn_stderr_drain(err);
        }

        let transport = Arc::new(Self {
            child: Arc::new(Mutex::new(Some(child))),
            stdin: Arc::new(Mutex::new(Some(stdin))),
        });
        Ok((transport, frame_rx))
    }
}

fn spawn_stdout_reader(
    stdout: tokio::process::ChildStdout,
    tx: mpsc::UnboundedSender<McpResult<String>>,
) {
    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break,
                Ok(_) => {
                    let trimmed = line.trim_end_matches(['\r', '\n']).to_string();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if tx.send(Ok(trimmed)).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(McpError::Io(e.to_string())));
                    break;
                }
            }
        }
    });
}

fn spawn_stderr_drain(stderr: tokio::process::ChildStderr) {
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    let trimmed = line.trim_end_matches(['\r', '\n']);
                    if !trimmed.is_empty() {
                        eprintln!("[mcp stderr] {trimmed}");
                    }
                }
            }
        }
    });
}

#[async_trait]
impl Transport for StdioTransport {
    async fn send(&self, frame: String) -> McpResult<()> {
        let mut guard = self.stdin.lock().await;
        let stdin = guard.as_mut().ok_or(McpError::Closed)?;
        stdin.write_all(frame.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;
        Ok(())
    }

    async fn shutdown(&self) -> McpResult<()> {
        if let Some(stdin) = self.stdin.lock().await.take() {
            drop(stdin);
        }
        if let Some(mut child) = self.child.lock().await.take() {
            let _ = child.start_kill();
            let _ = child.wait().await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn spawn_empty_command_fails() {
        let result = StdioTransport::spawn(StdioConfig {
            command: vec![],
            env: HashMap::new(),
        });
        assert!(matches!(result, Err(McpError::Transport(_))));
    }

    #[tokio::test]
    async fn echo_subprocess_roundtrip() {
        let (transport, mut rx) = StdioTransport::spawn(StdioConfig {
            command: vec!["cat".into()],
            env: HashMap::new(),
        })
        .expect("cat should be available");

        transport.send(r#"{"jsonrpc":"2.0","id":1}"#.to_string()).await.unwrap();
        let frame = tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv())
            .await
            .expect("frame should arrive")
            .expect("channel still open")
            .expect("frame is Ok");
        assert!(frame.contains("\"jsonrpc\""));
        transport.shutdown().await.unwrap();
    }
}
