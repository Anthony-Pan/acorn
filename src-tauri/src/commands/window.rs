use tauri::{AppHandle, Emitter, Manager};

#[tauri::command(rename_all = "camelCase")]
pub fn show_main(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub fn toggle_quick(app: AppHandle) -> Result<(), String> {
    toggle_quick_window(&app);
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub fn hide_quick(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("quick") {
        let _ = window.hide();
    }
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub fn open_settings(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = app.emit_to("main", "navigate", "settings");
    }
    Ok(())
}

pub fn toggle_quick_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("quick") {
        match window.is_visible() {
            Ok(true) => {
                let _ = window.hide();
            }
            _ => {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    }
}
