//! Desktop overlay windows: transparent, frameless, always-on-top surfaces that
//! float above other apps (and above fullscreen, on all Spaces) to give Acorn a
//! live desktop presence — the status capsule, pinned cards, the companion, and
//! transient reply summaries all share this one mechanism.
//!
//! A single `overlay.html` front-end entry is reused for every overlay; the
//! `?kind=` query string selects which React surface renders. Overlays run in
//! their own webview process, so they receive data from the main window over
//! Tauri events (never via the shared Zustand stores, which do not cross
//! webviews).

use tauri::webview::{WebviewWindow, WebviewWindowBuilder};
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, WebviewUrl, WindowEvent};

pub const NOTCH_LABEL: &str = "notch";
const NOTCH_W: f64 = 380.0;
const NOTCH_H: f64 = 72.0;
/// Gap below the menu bar; clears a notch comfortably on notched displays and
/// reads as "under the menu bar" on the rest.
const NOTCH_TOP_INSET: f64 = 8.0;

/// Build (once) a transparent always-on-top overlay window. Idempotent: if the
/// label already exists the existing window is returned untouched.
fn build_overlay(
    app: &AppHandle,
    label: &str,
    kind_query: &str,
    width: f64,
    height: f64,
    interactive_default: bool,
) -> tauri::Result<WebviewWindow> {
    if let Some(existing) = app.get_webview_window(label) {
        return Ok(existing);
    }

    let url = format!("overlay.html?{kind_query}");
    let window = WebviewWindowBuilder::new(app, label, WebviewUrl::App(url.into()))
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .shadow(false)
        .resizable(false)
        .visible(false)
        .inner_size(width, height)
        .build()?;

    // Portable "show on every Space" intent; the macOS path below also pins the
    // window above fullscreen apps via the native collection behavior.
    let _ = window.set_visible_on_all_workspaces(true);
    // Click-through by default — the front-end re-enables hit-testing only over
    // its interactive nodes via `set_overlay_interactive`.
    let _ = window.set_ignore_cursor_events(!interactive_default);

    #[cfg(target_os = "macos")]
    {
        elevate_overlay(&window);
        // Tauri can reset the window level after the first focus on release
        // builds; re-apply the native tweaks whenever the overlay gains focus.
        let reelevate = window.clone();
        window.on_window_event(move |event| {
            if let WindowEvent::Focused(true) = event {
                elevate_overlay(&reelevate);
            }
        });
    }

    Ok(window)
}

/// Raise an overlay above fullscreen windows and onto every Space using AppKit.
/// Best-effort: failures are non-fatal (the window still floats via Tauri's
/// portable always-on-top).
#[cfg(target_os = "macos")]
fn elevate_overlay(window: &WebviewWindow) {
    use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};

    // NSStatusWindowLevel (== 25) sits above normal and floating windows.
    const NS_STATUS_WINDOW_LEVEL: isize = 25;

    let Ok(ptr) = window.ns_window() else {
        return;
    };
    if ptr.is_null() {
        return;
    }

    let behavior = NSWindowCollectionBehavior::CanJoinAllSpaces
        | NSWindowCollectionBehavior::FullScreenAuxiliary
        | NSWindowCollectionBehavior::Stationary;

    // SAFETY: `ptr` is the live `NSWindow` Tauri owns for this label. We only
    // borrow it for the duration of these synchronous AppKit setters, which run
    // on the main thread (the setup hook, the `Focused` event handler, or a
    // `run_on_main_thread` dispatch). No ownership is taken or released.
    unsafe {
        let ns: &NSWindow = &*(ptr as *const NSWindow);
        ns.setLevel(NS_STATUS_WINDOW_LEVEL);
        ns.setCollectionBehavior(behavior);
    }
}

/// Center an overlay horizontally on its current monitor, `top_inset` logical
/// pixels below the top edge.
fn position_top_center(
    window: &WebviewWindow,
    width: f64,
    height: f64,
    top_inset: f64,
) -> tauri::Result<()> {
    if let Some(monitor) = window.current_monitor()? {
        // `Monitor::size()` is in physical pixels; convert to logical so the
        // math is correct on Retina displays.
        let scale = monitor.scale_factor();
        let monitor_width = monitor.size().width as f64 / scale;
        window.set_size(LogicalSize::new(width, height))?;
        window.set_position(LogicalPosition::new(
            (monitor_width - width) / 2.0,
            top_inset,
        ))?;
    }
    Ok(())
}

/// Create + position + show the notch status capsule. Safe to call repeatedly.
pub fn ensure_notch(app: &AppHandle) {
    match build_overlay(app, NOTCH_LABEL, "kind=notch", NOTCH_W, NOTCH_H, false) {
        Ok(window) => {
            let _ = position_top_center(&window, NOTCH_W, NOTCH_H, NOTCH_TOP_INSET);
            let _ = window.show();
            #[cfg(target_os = "macos")]
            elevate_overlay(&window);
        }
        Err(err) => eprintln!("acorn: failed to create notch overlay: {err}"),
    }
}

/// Toggle whether an overlay window swallows pointer events. The front-end calls
/// this on pointer enter/leave of its interactive zones so clicks elsewhere fall
/// through to the app behind the overlay.
#[tauri::command(rename_all = "camelCase")]
pub fn set_overlay_interactive(
    app: AppHandle,
    label: String,
    interactive: bool,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        window
            .set_ignore_cursor_events(!interactive)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub fn show_notch_overlay(app: AppHandle) -> Result<(), String> {
    let handle = app.clone();
    app.run_on_main_thread(move || ensure_notch(&handle))
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub fn hide_notch_overlay(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(NOTCH_LABEL) {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}
