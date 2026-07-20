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
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, WebviewUrl};

pub const NOTCH_LABEL: &str = "notch";
const NOTCH_W: f64 = 560.0;
const NOTCH_H: f64 = 64.0;
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
            if let tauri::WindowEvent::Focused(true) = event {
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
        // Monitor geometry is in physical pixels; convert to logical so the
        // math is correct on Retina displays. Offsetting by the monitor's own
        // origin keeps the overlay on the screen it lives on — without it,
        // secondary displays would push the window back onto the primary.
        let scale = monitor.scale_factor();
        let origin = monitor.position().to_logical::<f64>(scale);
        let monitor_width = monitor.size().width as f64 / scale;
        window.set_size(LogicalSize::new(width, height))?;
        window.set_position(LogicalPosition::new(
            origin.x + (monitor_width - width) / 2.0,
            origin.y + top_inset,
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

const PIN_W: f64 = 280.0;
const PIN_H: f64 = 132.0;

fn pin_label(pin_id: &str) -> String {
    format!("pin-{pin_id}")
}

/// Open (or focus) the desktop card window for a pin. Restores to a saved
/// logical position when present, otherwise lands in a top-right cascade.
pub fn open_pin(app: &AppHandle, pin_id: &str, x: Option<f64>, y: Option<f64>) {
    let label = pin_label(pin_id);
    let query = format!("kind=pin&pinId={pin_id}");
    match build_overlay(app, &label, &query, PIN_W, PIN_H, true) {
        Ok(window) => {
            match (x, y) {
                (Some(px), Some(py)) => {
                    let _ = window.set_position(LogicalPosition::new(px, py));
                }
                _ => {
                    let _ = position_pin_cascade(app, &window);
                }
            }
            let _ = window.show();
            #[cfg(target_os = "macos")]
            elevate_overlay(&window);
        }
        Err(err) => eprintln!("acorn: failed to open pin window: {err}"),
    }
}

/// Place a freshly opened pin near the top-right, nudged diagonally by the
/// number of pins already on screen so multiple cards don't stack exactly.
fn position_pin_cascade(app: &AppHandle, window: &WebviewWindow) -> tauri::Result<()> {
    let existing = app
        .webview_windows()
        .keys()
        .filter(|label| label.starts_with("pin-"))
        .count()
        .saturating_sub(1) as f64; // exclude the window we just built
    if let Some(monitor) = window.current_monitor()? {
        let scale = monitor.scale_factor();
        let origin = monitor.position().to_logical::<f64>(scale);
        let monitor_width = monitor.size().width as f64 / scale;
        let offset = (existing % 6.0) * 28.0;
        let x = origin.x + (monitor_width - PIN_W - 32.0 - offset).max(16.0);
        let y = origin.y + 72.0 + offset;
        window.set_position(LogicalPosition::new(x, y))?;
    }
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub fn open_pin_window(
    app: AppHandle,
    pin_id: String,
    x: Option<f64>,
    y: Option<f64>,
) -> Result<(), String> {
    let handle = app.clone();
    app.run_on_main_thread(move || open_pin(&handle, &pin_id, x, y))
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub fn close_pin_window(app: AppHandle, pin_id: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&pin_label(&pin_id)) {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub const SUMMARY_LABEL: &str = "summary";
const SUMMARY_W: f64 = 340.0;
const SUMMARY_H: f64 = 188.0;

/// Build + show the transient reply-summary overlay. The window is kept alive
/// across dismissals (hidden, not closed) so its `summary:show` listener never
/// misses an event after first creation.
fn ensure_summary(app: &AppHandle) {
    match build_overlay(
        app,
        SUMMARY_LABEL,
        "kind=summary",
        SUMMARY_W,
        SUMMARY_H,
        false,
    ) {
        Ok(window) => {
            let _ = position_top_center(&window, SUMMARY_W, SUMMARY_H, 12.0);
            let _ = window.show();
            #[cfg(target_os = "macos")]
            elevate_overlay(&window);
        }
        Err(err) => eprintln!("acorn: failed to create summary overlay: {err}"),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn show_summary_overlay(app: AppHandle) -> Result<(), String> {
    let handle = app.clone();
    app.run_on_main_thread(move || ensure_summary(&handle))
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub fn hide_summary_overlay(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(SUMMARY_LABEL) {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub const PET_LABEL: &str = "pet";
const PET_W: f64 = 108.0;
const PET_H: f64 = 124.0;
const PET_MARGIN: f64 = 24.0;

/// Anchor the companion to one of the four screen corners.
fn position_pet_corner(window: &WebviewWindow, corner: &str) -> tauri::Result<()> {
    if let Some(monitor) = window.current_monitor()? {
        let scale = monitor.scale_factor();
        let origin = monitor.position().to_logical::<f64>(scale);
        let monitor_width = monitor.size().width as f64 / scale;
        let monitor_height = monitor.size().height as f64 / scale;
        let left = origin.x + PET_MARGIN;
        let right = origin.x + monitor_width - PET_W - PET_MARGIN;
        let bottom = origin.y + monitor_height - PET_H - PET_MARGIN;
        let top = origin.y + PET_MARGIN + 28.0; // clear the menu bar / notch
        let (x, y) = match corner {
            "topLeft" => (left, top),
            "topRight" => (right, top),
            "bottomLeft" => (left, bottom),
            _ => (right, bottom), // bottomRight (default)
        };
        window.set_position(LogicalPosition::new(x, y))?;
    }
    Ok(())
}

/// Create + show the companion overlay in the bottom-right corner.
pub fn ensure_pet(app: &AppHandle) {
    match build_overlay(app, PET_LABEL, "kind=pet", PET_W, PET_H, true) {
        Ok(window) => {
            let _ = position_pet_corner(&window, "bottomRight");
            let _ = window.show();
            #[cfg(target_os = "macos")]
            elevate_overlay(&window);
        }
        Err(err) => eprintln!("acorn: failed to create pet overlay: {err}"),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn show_pet_overlay(app: AppHandle) -> Result<(), String> {
    let handle = app.clone();
    app.run_on_main_thread(move || ensure_pet(&handle))
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub fn hide_pet_overlay(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(PET_LABEL) {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub fn teleport_pet(app: AppHandle, corner: String) -> Result<(), String> {
    let handle = app.clone();
    app.run_on_main_thread(move || {
        if let Some(window) = handle.get_webview_window(PET_LABEL) {
            let _ = position_pet_corner(&window, &corner);
        }
    })
    .map_err(|e| e.to_string())
}

pub const APPROVAL_LABEL: &str = "approval";
const APPROVAL_W: f64 = 360.0;
const APPROVAL_H: f64 = 156.0;

/// Build the tool-approval card window hidden at boot, so its event listener is
/// registered well before any tool ever needs confirmation (no cold-start race).
pub fn ensure_approval_window(app: &AppHandle) {
    if let Err(err) = build_overlay(
        app,
        APPROVAL_LABEL,
        "kind=approval",
        APPROVAL_W,
        APPROVAL_H,
        true,
    ) {
        eprintln!("acorn: failed to create approval overlay: {err}");
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn show_approval_overlay(app: AppHandle) -> Result<(), String> {
    let handle = app.clone();
    app.run_on_main_thread(move || {
        ensure_approval_window(&handle);
        if let Some(window) = handle.get_webview_window(APPROVAL_LABEL) {
            // Sit just under the capsule.
            let _ = position_top_center(&window, APPROVAL_W, APPROVAL_H, 60.0);
            let _ = window.show();
            #[cfg(target_os = "macos")]
            elevate_overlay(&window);
        }
    })
    .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub fn hide_approval_overlay(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(APPROVAL_LABEL) {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}
