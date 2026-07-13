mod ai;
mod commands;
mod db;
mod error;
mod speech;

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, WindowEvent};
use tauri_plugin_deep_link::DeepLinkExt;

#[cfg(desktop)]
use std::str::FromStr;
#[cfg(desktop)]
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use commands::window::toggle_quick_window;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ));

    #[cfg(desktop)]
    let builder = builder.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(|app, shortcut, event| {
                if event.state() != ShortcutState::Pressed {
                    return;
                }
                if shortcut_matches(shortcut, commands::shortcut::PIN_SHORTCUT) {
                    let _ = app.emit(commands::shortcut::SHORTCUT_PIN_EVENT, ());
                    return;
                }
                if shortcut_matches(shortcut, commands::shortcut::SCREENSHOT_SHORTCUT) {
                    let _ = app.emit(commands::shortcut::SHORTCUT_SCREENSHOT_EVENT, ());
                    return;
                }
                if shortcut_matches(shortcut, commands::shortcut::PUSH_TO_TALK_SHORTCUT) {
                    let _ = app.emit(commands::shortcut::SHORTCUT_PUSH_TO_TALK_EVENT, ());
                    return;
                }
                if shortcut_matches(shortcut, commands::shortcut::QUICK_ASK_SHORTCUT) {
                    let _ = app.emit(commands::shortcut::SHORTCUT_QUICK_ASK_EVENT, ());
                    return;
                }
                if shortcut_matches(shortcut, commands::shortcut::KNOWLEDGE_CLOUD_SHORTCUT) {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                    let _ = app.emit(commands::shortcut::SHORTCUT_KNOWLEDGE_CLOUD_EVENT, ());
                    return;
                }
                toggle_quick_window(app);
            })
            .build(),
    );

    builder
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let db_path = app_data_dir.join("acorn.db");

            let database =
                tauri::async_runtime::block_on(async { db::Database::initialize(&db_path).await })?;

            app.manage(database);
            app.manage(commands::tool_approval::ApprovalBroker::default());

            #[cfg(target_os = "macos")]
            {
                // Run as a regular Dock app (Dock icon + ⌘-Tab switcher), not a
                // menubar-only accessory. The tray icon is kept alongside it.
                app.set_activation_policy(tauri::ActivationPolicy::Regular);
            }

            #[cfg(desktop)]
            {
                let stored = tauri::async_runtime::block_on(async {
                    commands::shortcut::load_shortcut(app.state::<db::Database>().inner()).await
                })
                .ok()
                .flatten()
                .unwrap_or_else(|| commands::shortcut::DEFAULT_SHORTCUT.to_string());
                let summon = Shortcut::from_str(&stored)
                    .or_else(|_| Shortcut::from_str(commands::shortcut::DEFAULT_SHORTCUT))?;
                register_global_shortcuts(app.handle(), summon)?;
                build_tray(app.handle(), &stored)?;
                wire_quick_window_blur(app.handle());
                wire_main_window_close_to_hide(app.handle());
                wire_deep_link(app.handle());
                commands::overlay::ensure_notch(app.handle());
                restore_pins(app.handle());
                maybe_spawn_pet(app.handle());
                commands::overlay::ensure_approval_window(app.handle());
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::session::create_session,
            commands::session::finalize_session,
            commands::session::get_session,
            commands::session::list_today_sessions,
            commands::session::list_recent_sessions,
            commands::task::insert_task,
            commands::task::update_task_status,
            commands::task::list_subtasks,
            commands::task::toggle_subtask,
            commands::task::set_task_schedule,
            commands::task::delete_task,
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::delete_setting,
            commands::provider_config::list_provider_configs,
            commands::provider_config::save_provider_config,
            commands::provider_config::touch_provider,
            commands::provider_config::get_active_provider,
            commands::provider_config::set_active_provider,
            commands::ai::list_providers,
            commands::ai::list_ollama_models,
            commands::ai::save_provider_credentials,
            commands::ai::delete_provider_credentials,
            commands::ai::has_provider_credentials,
            commands::ai::test_provider_connection,
            commands::ai::decompose,
            commands::conversation::create_conversation,
            commands::conversation::list_conversations,
            commands::conversation::get_conversation,
            commands::conversation::delete_conversation,
            commands::conversation::rename_conversation,
            commands::conversation::set_conversation_provider,
            commands::conversation::list_conversation_cloud,
            commands::conversation::set_conversation_favorite,
            commands::conversation::set_conversation_archived,
            commands::chat::chat,
            commands::window::show_main,
            commands::window::toggle_quick,
            commands::window::hide_quick,
            commands::window::open_settings,
            commands::overlay::set_overlay_interactive,
            commands::overlay::show_notch_overlay,
            commands::overlay::hide_notch_overlay,
            commands::overlay::open_pin_window,
            commands::overlay::close_pin_window,
            commands::overlay::show_summary_overlay,
            commands::overlay::hide_summary_overlay,
            commands::overlay::show_pet_overlay,
            commands::overlay::hide_pet_overlay,
            commands::overlay::teleport_pet,
            commands::overlay::show_approval_overlay,
            commands::overlay::hide_approval_overlay,
            commands::pin::list_pins,
            commands::pin::get_pin,
            commands::pin::create_pin,
            commands::pin::update_pin_position,
            commands::pin::delete_pin,
            commands::shortcut::get_summon_shortcut,
            commands::shortcut::set_summon_shortcut,
            commands::shortcut::reset_summon_shortcut,
            commands::speech::list_speech_providers,
            commands::speech::list_speech_provider_configs,
            commands::speech::save_speech_provider_config,
            commands::speech::get_active_speech_provider,
            commands::speech::set_active_speech_provider,
            commands::speech::transcribe_audio,
            commands::activity::record_activity,
            commands::activity::list_recent_activity,
            commands::activity::clear_activity,
            commands::activity::build_daily_brief,
            commands::updater::check_for_update,
            commands::capture::capture_primary_screen,
            commands::canvas::list_canvases,
            commands::canvas::get_canvas,
            commands::canvas::create_canvas,
            commands::canvas::update_canvas,
            commands::canvas::delete_canvas,
            commands::search::search_index,
            commands::tool_approval::respond_tool_approval,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, _event| {
            // macOS: the main window hides (instead of quitting) on close, so a
            // Dock-icon click with no visible window must reopen it — otherwise
            // a regular Dock app would have no way back to its window.
            #[cfg(target_os = "macos")]
            {
                if let tauri::RunEvent::Reopen {
                    has_visible_windows: false,
                    ..
                } = &_event
                {
                    if let Some(window) = _app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        });
}

/// Register the summon shortcut plus the fixed extras. `set_summon_shortcut`
/// re-runs this after its `unregister_all`, so every shortcut the app owns must
/// be registered here — a shortcut registered anywhere else would silently die
/// the first time the user rebinds the summon key.
#[cfg(desktop)]
pub(crate) fn register_global_shortcuts(
    app: &tauri::AppHandle,
    summon: Shortcut,
) -> anyhow::Result<()> {
    let extras = [
        commands::shortcut::PIN_SHORTCUT,
        commands::shortcut::SCREENSHOT_SHORTCUT,
        commands::shortcut::PUSH_TO_TALK_SHORTCUT,
        commands::shortcut::QUICK_ASK_SHORTCUT,
        commands::shortcut::KNOWLEDGE_CLOUD_SHORTCUT,
    ];
    let gs = app.global_shortcut();
    gs.register(summon)?;
    for raw in extras {
        if let Ok(parsed) = Shortcut::from_str(raw) {
            if let Err(err) = gs.register(parsed) {
                eprintln!("acorn: could not register shortcut '{raw}': {err}");
            }
        }
    }
    Ok(())
}

#[cfg(desktop)]
fn shortcut_matches(target: &Shortcut, spec: &str) -> bool {
    Shortcut::from_str(spec)
        .map(|expected| expected == *target)
        .unwrap_or(false)
}

fn build_tray(app: &tauri::AppHandle, summon_shortcut: &str) -> anyhow::Result<()> {
    let menu = tray_menu(app, summon_shortcut)?;

    let icon = app
        .default_window_icon()
        .expect("acorn bundle ships with a default window icon")
        .clone();

    TrayIconBuilder::with_id("acorn-tray")
        .menu(&menu)
        .icon(icon)
        .icon_as_template(true)
        .on_menu_event(handle_tray_menu)
        .build(app)?;

    Ok(())
}

fn tray_menu(app: &tauri::AppHandle, summon_shortcut: &str) -> anyhow::Result<Menu<tauri::Wry>> {
    let summon_label = format!("Summon  {}", format_shortcut_label(summon_shortcut));
    let show = MenuItem::with_id(app, "show", "Show Acorn", true, None::<&str>)?;
    let summon = MenuItem::with_id(app, "summon", summon_label, true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Acorn", true, None::<&str>)?;
    Ok(Menu::with_items(
        app,
        &[&show, &summon, &settings, &separator, &quit],
    )?)
}

/// Refresh the tray's "Summon" entry after a rebind so the menu never
/// advertises a shortcut that is no longer registered.
pub(crate) fn update_tray_summon_shortcut(app: &tauri::AppHandle, shortcut: &str) {
    if let Some(tray) = app.tray_by_id("acorn-tray") {
        match tray_menu(app, shortcut) {
            Ok(menu) => {
                if let Err(err) = tray.set_menu(Some(menu)) {
                    eprintln!("acorn: could not update tray menu: {err}");
                }
            }
            Err(err) => eprintln!("acorn: could not rebuild tray menu: {err}"),
        }
    }
}

/// "CmdOrCtrl+Shift+KeyA" → "⌘⇧A" for menu labels.
fn format_shortcut_label(raw: &str) -> String {
    raw.split('+')
        .map(|part| match part {
            "CmdOrCtrl" | "CommandOrControl" | "Cmd" | "Command" | "Super" | "Meta" => {
                "⌘".to_string()
            }
            "Ctrl" | "Control" => "⌃".to_string(),
            "Shift" => "⇧".to_string(),
            "Alt" | "Option" => "⌥".to_string(),
            p if p.starts_with("Key") => p[3..].to_string(),
            p if p.starts_with("Digit") => p[5..].to_string(),
            p => p.to_string(),
        })
        .collect()
}

#[cfg(desktop)]
fn wire_quick_window_blur(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("quick") {
        let window_for_handler = window.clone();
        window.on_window_event(move |event| {
            if let WindowEvent::Focused(false) = event {
                let _ = window_for_handler.hide();
            }
        });
    }
}

#[cfg(desktop)]
fn wire_main_window_close_to_hide(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let window_for_handler = window.clone();
        window.on_window_event(move |event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window_for_handler.hide();
            }
        });
    }
}

#[cfg(desktop)]
fn wire_deep_link(app: &tauri::AppHandle) {
    let handle = app.clone();
    app.deep_link().on_open_url(move |event| {
        let handle = handle.clone();
        let parsed: Vec<(String, Option<String>)> = event
            .urls()
            .iter()
            .map(|u| (u.to_string(), u.host_str().map(|s| s.to_string())))
            .collect();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            if let Some(window) = handle.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
            for (raw, host) in parsed {
                if matches!(host.as_deref(), Some("summon")) {
                    toggle_quick_window(&handle);
                }
                let _ = handle.emit("deep-link", raw);
            }
        });
    });
}

#[cfg(desktop)]
fn restore_pins(app: &tauri::AppHandle) {
    let pins = tauri::async_runtime::block_on(async {
        commands::pin::load_pins(app.state::<db::Database>().inner()).await
    })
    .unwrap_or_default();
    for pin in pins {
        commands::overlay::open_pin(app, &pin.id, pin.x, pin.y);
    }
}

#[cfg(desktop)]
fn maybe_spawn_pet(app: &tauri::AppHandle) {
    let hidden = tauri::async_runtime::block_on(async {
        commands::settings::load_setting(app.state::<db::Database>().inner(), "pet-hidden").await
    })
    .ok()
    .flatten();
    if hidden.as_deref() != Some("true") {
        commands::overlay::ensure_pet(app);
    }
}

fn handle_tray_menu(app: &tauri::AppHandle, event: tauri::menu::MenuEvent) {
    match event.id.as_ref() {
        "show" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "summon" => toggle_quick_window(app),
        "settings" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = app.emit_to("main", "navigate", "settings");
            }
        }
        "quit" => app.exit(0),
        _ => {}
    }
}
