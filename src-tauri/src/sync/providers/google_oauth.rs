//! Google OAuth 2.0 for an installed/desktop app: Authorization Code + PKCE
//! (S256) with a `127.0.0.1` loopback redirect. The system browser handles
//! consent; a one-shot local TCP listener catches the redirect. Only the
//! refresh token is persisted (in the keychain); access tokens are minted on
//! demand and kept in memory.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use reqwest::Url;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::sync::error::{SyncError, SyncResult};
use crate::sync::types::SyncProviderKind;

const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v3/userinfo";

/// Shipped-client model: the Google Cloud project is Acorn's, injected at build
/// time. Google treats the desktop client_secret as non-confidential — PKCE is
/// the real protection. Absent at build → connect returns a clear error.
fn client_credentials() -> SyncResult<(&'static str, &'static str)> {
    let id = option_env!("ACORN_GOOGLE_CLIENT_ID").unwrap_or("");
    let secret = option_env!("ACORN_GOOGLE_CLIENT_SECRET").unwrap_or("");
    if id.is_empty() {
        return Err(SyncError::OAuth(
            "Acorn was built without Google credentials (set ACORN_GOOGLE_CLIENT_ID / \
             ACORN_GOOGLE_CLIENT_SECRET at build time)"
                .into(),
        ));
    }
    Ok((id, secret))
}

fn scopes_for(kind: SyncProviderKind) -> &'static str {
    match kind {
        SyncProviderKind::GoogleCalendar => "openid email https://www.googleapis.com/auth/calendar",
        SyncProviderKind::GoogleTasks => "openid email https://www.googleapis.com/auth/tasks",
        // Apple providers never reach here.
        _ => "openid email",
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
}

/// The credentials produced by a successful consent. Only the refresh token is
/// persisted; the initial access token is discarded (providers mint their own).
pub struct GoogleTokens {
    pub refresh_token: String,
    pub email: String,
}

fn random_urlsafe(bytes: usize) -> SyncResult<String> {
    let mut buf = vec![0u8; bytes];
    getrandom::getrandom(&mut buf).map_err(|e| SyncError::OAuth(format!("rng failure: {e}")))?;
    Ok(URL_SAFE_NO_PAD.encode(buf))
}

fn s256_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(hasher.finalize())
}

fn open_in_browser(url: &str) -> SyncResult<()> {
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut c = std::process::Command::new("open");
        c.arg(url);
        c
    };
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut c = std::process::Command::new("cmd");
        c.args(["/C", "start", "", url]);
        c
    };
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = {
        let mut c = std::process::Command::new("xdg-open");
        c.arg(url);
        c
    };
    command
        .spawn()
        .map_err(|e| SyncError::OAuth(format!("could not open browser: {e}")))?;
    Ok(())
}

/// Run the full consent dance and return the resulting tokens + account email.
pub async fn run_oauth_flow(kind: SyncProviderKind) -> SyncResult<GoogleTokens> {
    let (client_id, client_secret) = client_credentials()?;
    let verifier = random_urlsafe(32)?;
    let challenge = s256_challenge(&verifier);
    let state = random_urlsafe(16)?;

    let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
    let port = listener.local_addr()?.port();
    let redirect_uri = format!("http://127.0.0.1:{port}");

    let auth_url = Url::parse_with_params(
        AUTH_URL,
        &[
            ("client_id", client_id),
            ("redirect_uri", redirect_uri.as_str()),
            ("response_type", "code"),
            ("scope", scopes_for(kind)),
            ("code_challenge", challenge.as_str()),
            ("code_challenge_method", "S256"),
            ("state", state.as_str()),
            ("access_type", "offline"),
            ("prompt", "consent"),
        ],
    )
    .map_err(|e| SyncError::OAuth(e.to_string()))?;

    open_in_browser(auth_url.as_str())?;

    let code = capture_code(&listener, &state, Duration::from_secs(180)).await?;

    let token = exchange_code(client_id, client_secret, &code, &verifier, &redirect_uri).await?;

    let refresh_token = token.refresh_token.ok_or_else(|| {
        SyncError::OAuth(
            "Google did not return a refresh token — revoke Acorn's access in your Google \
             account and reconnect."
                .into(),
        )
    })?;
    let email = fetch_email(&token.access_token)
        .await
        .unwrap_or_else(|_| "Google account".to_string());

    Ok(GoogleTokens {
        refresh_token,
        email,
    })
}

/// Accept the browser's redirect on the loopback socket and extract `code`,
/// verifying `state`. Ignores incidental requests (favicon, etc.) until the
/// authorization response arrives or the timeout elapses.
async fn capture_code(
    listener: &TcpListener,
    expected_state: &str,
    timeout: Duration,
) -> SyncResult<String> {
    let work = async {
        loop {
            let (mut stream, _) = listener.accept().await?;
            let mut buf = vec![0u8; 8192];
            let n = stream.read(&mut buf).await?;
            let request = String::from_utf8_lossy(&buf[..n]);
            let path = request
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .unwrap_or("");

            let parsed = Url::parse(&format!("http://127.0.0.1{path}")).ok();
            let (mut code, mut got_state, mut oauth_err) = (None, None, None);
            if let Some(url) = &parsed {
                for (key, value) in url.query_pairs() {
                    match key.as_ref() {
                        "code" => code = Some(value.into_owned()),
                        "state" => got_state = Some(value.into_owned()),
                        "error" => oauth_err = Some(value.into_owned()),
                        _ => {}
                    }
                }
            }

            if code.is_none() && oauth_err.is_none() {
                // Not the redirect we're waiting for; politely dismiss and keep listening.
                let _ = write_response(&mut stream, "404 Not Found", "Waiting for Google…").await;
                continue;
            }

            let _ = write_response(
                &mut stream,
                "200 OK",
                "Acorn is connected. You can close this tab and return to the app.",
            )
            .await;

            if let Some(err) = oauth_err {
                return Err(SyncError::OAuth(format!("consent was denied: {err}")));
            }
            if got_state.as_deref() != Some(expected_state) {
                return Err(SyncError::OAuth("state mismatch on redirect".into()));
            }
            return code.ok_or_else(|| SyncError::OAuth("no authorization code returned".into()));
        }
    };

    match tokio::time::timeout(timeout, work).await {
        Ok(result) => result,
        Err(_) => Err(SyncError::OAuth(
            "timed out waiting for Google consent".into(),
        )),
    }
}

async fn write_response(
    stream: &mut tokio::net::TcpStream,
    status: &str,
    message: &str,
) -> std::io::Result<()> {
    let body = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Acorn</title></head>\
         <body style=\"font-family:-apple-system,system-ui,sans-serif;text-align:center;\
         margin-top:20vh;color:#333\"><h2>🌰 {message}</h2></body></html>"
    );
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await
}

async fn exchange_code(
    client_id: &str,
    client_secret: &str,
    code: &str,
    verifier: &str,
    redirect_uri: &str,
) -> SyncResult<TokenResponse> {
    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("code", code),
        ("code_verifier", verifier),
        ("grant_type", "authorization_code"),
        ("redirect_uri", redirect_uri),
    ];
    let resp = reqwest::Client::new()
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(SyncError::OAuth(format!(
            "token exchange failed ({status}): {body}"
        )));
    }
    Ok(resp.json().await?)
}

/// Exchange the long-lived refresh token for a fresh access token.
pub async fn mint_access_token(refresh_token: &str) -> SyncResult<String> {
    let (client_id, client_secret) = client_credentials()?;
    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];
    let resp = reqwest::Client::new()
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(SyncError::Unauthorized(format!(
            "refresh failed ({status}): {body}"
        )));
    }
    let token: TokenResponse = resp.json().await?;
    Ok(token.access_token)
}

async fn fetch_email(access_token: &str) -> SyncResult<String> {
    let resp = reqwest::Client::new()
        .get(USERINFO_URL)
        .bearer_auth(access_token)
        .send()
        .await?;
    let value: serde_json::Value = resp.json().await?;
    Ok(value
        .get("email")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("Google account")
        .to_string())
}
