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

pub const SHORTCUT_PIN_EVENT: &str = "shortcut:pin-response";
pub const SHORTCUT_SCREENSHOT_EVENT: &str = "shortcut:screenshot";
pub const SHORTCUT_PUSH_TO_TALK_EVENT: &str = "shortcut:push-to-talk";
pub const SHORTCUT_QUICK_ASK_EVENT: &str = "shortcut:quick-ask";

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
    // Validate up front so an unparseable hotkey is rejected before we touch
    // the live registration or persist anything.
    Shortcut::from_str(trimmed)
        .map_err(|e| AppError::InvalidInput(format!("invalid shortcut '{trimmed}': {e}")))?;

    // Re-register the summon hotkey *and* every helper shortcut together.
    // `unregister_all` wipes the lot, so registering only the summon here would
    // silently kill pin/screenshot/push-to-talk/quick-ask until the next launch.
    #[cfg(desktop)]
    register_all_shortcuts(&app, trimmed)
        .map_err(|e| AppError::InvalidInput(format!("could not register '{trimmed}': {e}")))?;
    #[cfg(not(desktop))]
    let _ = &app;

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

/// Register the summon hotkey plus every built-in helper shortcut in one shot.
///
/// Used both at startup and whenever the user changes the summon hotkey. It
/// clears any existing bindings first so re-running it never double-registers
/// or strands the helper shortcuts. Helper-shortcut failures are non-fatal
/// (logged, not propagated) so one bad/taken binding can't take the rest down;
/// the summon registration result is returned so the caller can surface it.
#[cfg(desktop)]
pub fn register_all_shortcuts(app: &AppHandle, summon_str: &str) -> Result<(), String> {
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();

    // Register helpers first so a bad summon hotkey never starves them.
    for raw in [
        PIN_SHORTCUT,
        SCREENSHOT_SHORTCUT,
        PUSH_TO_TALK_SHORTCUT,
        QUICK_ASK_SHORTCUT,
    ] {
        match Shortcut::from_str(raw) {
            Ok(parsed) => {
                if let Err(err) = gs.register(parsed) {
                    eprintln!("acorn: could not register shortcut '{raw}': {err}");
                }
            }
            Err(err) => eprintln!("acorn: invalid built-in shortcut '{raw}': {err}"),
        }
    }

    let summon = Shortcut::from_str(summon_str)
        .or_else(|_| Shortcut::from_str(DEFAULT_SHORTCUT))
        .map_err(|e| format!("invalid shortcut: {e}"))?;
    gs.register(summon).map_err(|e| e.to_string())
}
