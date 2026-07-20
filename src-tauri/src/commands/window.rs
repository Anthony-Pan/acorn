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
                #[cfg(target_os = "macos")]
                activate_app(app);
            }
        }
    }
}

/// Bring the whole application forward so a freshly-shown borderless window can
/// actually take keyboard focus. Acorn runs as a macOS *accessory* app (no Dock
/// icon); for accessory apps, `show()` + `set_focus()` alone often leave the
/// quick window visible-but-not-key when another app is frontmost. Activating
/// the NSApplication on the main thread makes the summoned window key.
#[cfg(target_os = "macos")]
fn activate_app(app: &AppHandle) {
    let _ = app.run_on_main_thread(|| {
        // SAFETY: `run_on_main_thread` guarantees we are on the main thread,
        // which every NSApplication call requires.
        if let Some(mtm) = objc2::MainThreadMarker::new() {
            let ns_app = objc2_app_kit::NSApplication::sharedApplication(mtm);
            #[allow(deprecated)]
            unsafe {
                ns_app.activateIgnoringOtherApps(true);
            }
        }
    });
}
