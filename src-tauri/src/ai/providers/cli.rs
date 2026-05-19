//! CLI-based AI provider: shells out to a locally installed agent CLI
//! (`claude`, `codex`, `gemini`, `hermes`, ...) and reads its stdout.
//!
//! Each CLI has its own non-interactive flag and output wrapping; the
//! per-id invocation table at the bottom of this file says what command
//! to spawn, which args wrap the prompt, and how to extract the model's
//! reply from stdout.
//!
//! Authentication is the CLI's own concern — Acorn only needs the
//! binary on the user's `PATH`. No API key, no custom endpoint, no
//! keychain entry.

use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::time::timeout;

use crate::ai::error::{ProviderError, ProviderResult};
use crate::ai::metadata::ProviderMetadata;
use crate::ai::prompts::DECOMPOSE_SYSTEM_PROMPT;
use crate::ai::provider::{emit_decomposed, format_user_input, strip_code_fences, Provider};
use crate::ai::types::{DecomposeEvent, DecomposeRequest, DecomposeResponse};

#[derive(Debug, Clone, Copy)]
pub struct CliInvocation {
    pub command: &'static str,
    pub pre_args: &'static [&'static str],
    pub post_args: &'static [&'static str],
    pub output_parser: OutputParser,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputParser {
    PlainText,
    JsonField(&'static str),
}

pub struct CliProvider {
    metadata: ProviderMetadata,
    invocation: CliInvocation,
}

impl CliProvider {
    pub fn new(metadata: ProviderMetadata, invocation: CliInvocation) -> Self {
        Self {
            metadata,
            invocation,
        }
    }
}

// Five-minute cap on a single CLI invocation. The Acorn UI blocks the
// user until decompose returns, so a runaway agent CLI shouldn't hang
// forever.
const CLI_TIMEOUT: Duration = Duration::from_secs(300);

#[async_trait]
impl Provider for CliProvider {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn decompose(
        &self,
        request: DecomposeRequest,
        events: mpsc::Sender<DecomposeEvent>,
    ) -> ProviderResult<DecomposeResponse> {
        let user_content = format_user_input(&request);
        let prompt = format!("{DECOMPOSE_SYSTEM_PROMPT}\n\n{user_content}");

        let _ = events
            .send(DecomposeEvent::Progress { received_chars: 0 })
            .await;

        let stdout = run_cli(&self.invocation, &prompt).await?;

        let content = match self.invocation.output_parser {
            OutputParser::PlainText => stdout,
            OutputParser::JsonField(field) => extract_json_field(&stdout, field)?,
        };

        let _ = events
            .send(DecomposeEvent::Progress {
                received_chars: content.len(),
            })
            .await;

        let json = strip_code_fences(&content);
        let response: DecomposeResponse = serde_json::from_str(json).map_err(|e| {
            ProviderError::InvalidResponse(format!(
                "{} CLI did not return DecomposeResponse JSON: {e}\n--- stdout ---\n{content}",
                self.invocation.command
            ))
        })?;

        emit_decomposed(&events, &response).await?;
        Ok(response)
    }

    async fn validate_credentials(&self) -> ProviderResult<()> {
        let out = Command::new(self.invocation.command)
            .arg("--version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                ProviderError::MissingCredentials(format!(
                    "{} CLI not found on PATH ({}). Install it and re-test.",
                    self.invocation.command, e
                ))
            })?;
        if !out.status.success() {
            return Err(ProviderError::ProviderResponse(format!(
                "`{} --version` returned status {}",
                self.invocation.command, out.status
            )));
        }
        Ok(())
    }
}

async fn run_cli(inv: &CliInvocation, prompt: &str) -> ProviderResult<String> {
    let mut cmd = Command::new(inv.command);
    cmd.args(inv.pre_args);
    cmd.arg(prompt);
    cmd.args(inv.post_args);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let child = cmd.spawn().map_err(|e| {
        ProviderError::MissingCredentials(format!(
            "{} CLI not found on PATH ({}). Install it and try again.",
            inv.command, e
        ))
    })?;

    let output = timeout(CLI_TIMEOUT, child.wait_with_output())
        .await
        .map_err(|_| {
            ProviderError::Network(format!(
                "{} CLI timed out after {} seconds",
                inv.command,
                CLI_TIMEOUT.as_secs()
            ))
        })?
        .map_err(|e| ProviderError::Network(format!("{}: {e}", inv.command)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ProviderError::ProviderResponse(format!(
            "{} CLI exited with status {}: {}",
            inv.command,
            output.status,
            stderr.trim()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn extract_json_field(stdout: &str, field: &str) -> ProviderResult<String> {
    let trimmed = stdout.trim();

    // Try whole stdout first, then each `{...}` line from the bottom up —
    // some CLIs emit one JSON object, claude code emits one JSON per line
    // and the final answer is the last such line.
    let mut candidates: Vec<&str> = vec![trimmed];
    candidates.extend(
        trimmed
            .lines()
            .rev()
            .map(str::trim)
            .filter(|l| !l.is_empty() && l.starts_with('{')),
    );

    for candidate in candidates {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(candidate) else {
            continue;
        };
        if let Some(s) = value.get(field).and_then(|v| v.as_str()) {
            return Ok(s.to_string());
        }
    }

    Err(ProviderError::InvalidResponse(format!(
        "CLI output did not contain JSON field `{field}`. Raw stdout:\n{stdout}"
    )))
}

pub fn invocation_for(provider_id: &str) -> Option<CliInvocation> {
    match provider_id {
        "claude_cli" => Some(CliInvocation {
            command: "claude",
            pre_args: &["-p"],
            post_args: &["--output-format", "json"],
            output_parser: OutputParser::JsonField("result"),
        }),
        "codex_cli" => Some(CliInvocation {
            command: "codex",
            pre_args: &["exec"],
            post_args: &[],
            output_parser: OutputParser::PlainText,
        }),
        "gemini_cli" => Some(CliInvocation {
            command: "gemini",
            pre_args: &["-p"],
            post_args: &["--output-format", "json"],
            output_parser: OutputParser::JsonField("response"),
        }),
        "hermes_cli" => Some(CliInvocation {
            command: "hermes",
            pre_args: &["chat", "-q"],
            post_args: &["--quiet"],
            output_parser: OutputParser::PlainText,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_top_level_field() {
        let stdout = r#"{"response": "hello", "stats": {}}"#;
        assert_eq!(extract_json_field(stdout, "response").unwrap(), "hello");
    }

    #[test]
    fn extracts_field_from_last_jsonl_line() {
        let stdout = "logging line\n{\"result\":\"final answer\"}\n";
        assert_eq!(
            extract_json_field(stdout, "result").unwrap(),
            "final answer"
        );
    }

    #[test]
    fn missing_field_errors_clearly() {
        let stdout = r#"{"unrelated": 1}"#;
        let err = extract_json_field(stdout, "result").unwrap_err();
        assert!(err
            .to_string()
            .contains("did not contain JSON field `result`"));
    }

    #[test]
    fn invocation_table_covers_all_four_clis() {
        for id in ["claude_cli", "codex_cli", "gemini_cli", "hermes_cli"] {
            assert!(invocation_for(id).is_some(), "missing invocation for {id}");
        }
        assert!(invocation_for("unknown").is_none());
    }
}
