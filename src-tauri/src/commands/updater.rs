use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::error::{AppError, AppResult};

const RELEASES_URL: &str = "https://api.github.com/repos/onyxcraft/acorn/releases/latest";

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    name: Option<String>,
    body: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAvailable {
    pub current_version: String,
    pub latest_version: String,
    pub release_name: Option<String>,
    pub release_notes: Option<String>,
    pub release_url: String,
}

#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> AppResult<Option<UpdateAvailable>> {
    let current = app.package_info().version.to_string();

    let client = reqwest::Client::builder()
        .user_agent(concat!("acorn/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| AppError::InvalidInput(format!("could not build http client: {e}")))?;

    let response = client.get(RELEASES_URL).send().await;
    let release: GithubRelease = match response {
        Ok(resp) if resp.status().is_success() => resp
            .json()
            .await
            .map_err(|e| AppError::InvalidInput(format!("invalid release payload: {e}")))?,
        Ok(_) | Err(_) => return Ok(None),
    };

    let latest = release.tag_name.trim_start_matches('v').to_string();
    if latest == current || !is_newer(&latest, &current) {
        return Ok(None);
    }

    Ok(Some(UpdateAvailable {
        current_version: current,
        latest_version: latest,
        release_name: release.name,
        release_notes: release.body,
        release_url: release.html_url,
    }))
}

fn is_newer(candidate: &str, current: &str) -> bool {
    let parse = |s: &str| -> Vec<u32> {
        s.split('.')
            .map(|p| {
                p.chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse::<u32>()
                    .unwrap_or(0)
            })
            .collect()
    };
    let a = parse(candidate);
    let b = parse(current);
    for i in 0..a.len().max(b.len()) {
        let av = a.get(i).copied().unwrap_or(0);
        let bv = b.get(i).copied().unwrap_or(0);
        if av > bv {
            return true;
        }
        if av < bv {
            return false;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::is_newer;

    #[test]
    fn detects_patch_bump() {
        assert!(is_newer("1.0.1", "1.0.0"));
    }

    #[test]
    fn detects_minor_bump() {
        assert!(is_newer("1.1.0", "1.0.9"));
    }

    #[test]
    fn rejects_same_version() {
        assert!(!is_newer("1.2.3", "1.2.3"));
    }

    #[test]
    fn rejects_older_candidate() {
        assert!(!is_newer("1.0.0", "1.0.1"));
    }

    #[test]
    fn ignores_trailing_prerelease_suffix() {
        assert!(is_newer("1.2.0-beta1", "1.1.9"));
    }
}
