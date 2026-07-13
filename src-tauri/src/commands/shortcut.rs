use std::str::FromStr;

use tauri::{AppHandle, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::db::Database;
use crate::error::{AppError, AppResult};

pub const SHORTCUT_SETTING_KEY: &str = "summon_shortcut";
pub const DEFAULT_SHORTCUT: &str = "CmdOrCtrl+Shift+KeyA";

pub const PIN_SHORTCUT: &str = "CmdOrCtrl+Shift+KeyP";
pub const SCREENSHOT_SHORTCUT: &str = "CmdOrCtrl+Shift+KeyJ";
pub const PUSH_TO_TALK_SHORTCUT: &str = "CmdOrCtrl+Shift+KeyV";
pub const QUICK_ASK_SHORTCUT: &str = "CmdOrCtrl+Shift+Space";
pub const KNOWLEDGE_CLOUD_SHORTCUT: &str = "CmdOrCtrl+Shift+KeyG";

pub const SHORTCUT_PIN_EVENT: &str = "shortcut:pin-response";
pub const SHORTCUT_SCREENSHOT_EVENT: &str = "shortcut:screenshot";
pub const SHORTCUT_PUSH_TO_TALK_EVENT: &str = "shortcut:push-to-talk";
pub const SHORTCUT_QUICK_ASK_EVENT: &str = "shortcut:quick-ask";
pub const SHORTCUT_KNOWLEDGE_CLOUD_EVENT: &str = "shortcut:knowledge-cloud";

#[tauri::command(rename_all = "camelCase")]
pub async fn get_summon_shortcut(db: State<'_, Database>) -> AppResult<String> {
    Ok(load_shortcut(&db)
        .await?
        .unwrap_or_else(|| DEFAULT_SHORTCUT.to_string()))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn set_summon_shortcut(
    app: AppHandle,
    db: State<'_, Database>,
    shortcut: String,
) -> AppResult<()> {
    let trimmed = shortcut.trim();
    let parsed = Shortcut::from_str(trimmed)
        .map_err(|e| AppError::InvalidInput(format!("invalid shortcut '{trimmed}': {e}")))?;

    #[cfg(desktop)]
    {
        app.global_shortcut()
            .unregister_all()
            .map_err(|e| AppError::InvalidInput(format!("could not unregister previous: {e}")))?;
        // unregister_all also dropped the pin/screenshot/push-to-talk/quick-ask/
        // cloud extras, so re-register the full set — not just the summon key.
        crate::register_global_shortcuts(&app, parsed)
            .map_err(|e| AppError::InvalidInput(format!("could not register: {e}")))?;
        crate::update_tray_summon_shortcut(&app, trimmed);
    }
    #[cfg(not(desktop))]
    let _ = (app, parsed);

    sqlx::query(
        "INSERT INTO settings (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(SHORTCUT_SETTING_KEY)
    .bind(trimmed)
    .execute(db.pool())
    .await?;

    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn reset_summon_shortcut(app: AppHandle, db: State<'_, Database>) -> AppResult<String> {
    set_summon_shortcut(app, db, DEFAULT_SHORTCUT.to_string()).await?;
    Ok(DEFAULT_SHORTCUT.to_string())
}

pub async fn load_shortcut(db: &Database) -> AppResult<Option<String>> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
        .bind(SHORTCUT_SETTING_KEY)
        .fetch_optional(db.pool())
        .await?;
    Ok(row.map(|(v,)| v))
}
