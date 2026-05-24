use chrono::Utc;
use tauri::{AppHandle, Manager};
use xcap::Monitor;

use crate::error::{AppError, AppResult};

#[tauri::command]
pub async fn capture_primary_screen(app: AppHandle) -> AppResult<String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::InvalidInput(format!("could not resolve app data dir: {e}")))?
        .join("screenshots");
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| AppError::InvalidInput(format!("could not create screenshots dir: {e}")))?;

    let stamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let path = dir.join(format!("acorn-{stamp}.png"));
    let path_for_capture = path.clone();

    tokio::task::spawn_blocking(move || -> Result<(), AppError> {
        let monitors = Monitor::all()
            .map_err(|e| AppError::InvalidInput(format!("could not enumerate monitors: {e}")))?;
        let monitor = monitors
            .into_iter()
            .next()
            .ok_or_else(|| AppError::InvalidInput("no monitors available to capture".into()))?;
        let image = monitor
            .capture_image()
            .map_err(|e| AppError::InvalidInput(format!("capture_image failed: {e}")))?;
        image
            .save(&path_for_capture)
            .map_err(|e| AppError::InvalidInput(format!("could not write PNG: {e}")))?;
        Ok(())
    })
    .await
    .map_err(|e| AppError::InvalidInput(format!("capture task panicked: {e}")))??;

    Ok(path.to_string_lossy().to_string())
}
