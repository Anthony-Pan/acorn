mod ai;
mod commands;
mod db;
mod error;

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, WindowEvent};

#[cfg(desktop)]
use std::str::FromStr;
#[cfg(desktop)]
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use commands::window::toggle_quick_window;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ));

    #[cfg(desktop)]
    let builder = builder.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(|app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    toggle_quick_window(app);
                }
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

            #[cfg(desktop)]
            {
                let stored = tauri::async_runtime::block_on(async {
                    commands::shortcut::load_shortcut(app.state::<db::Database>().inner()).await
                })
                .ok()
                .flatten()
                .unwrap_or_else(|| commands::shortcut::DEFAULT_SHORTCUT.to_string());
                register_global_shortcuts(app.handle(), &stored)?;
                build_tray(app.handle())?;
                wire_quick_window_blur(app.handle());
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
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::delete_setting,
            commands::provider_config::list_provider_configs,
            commands::provider_config::save_provider_config,
            commands::provider_config::touch_provider,
            commands::provider_config::get_active_provider,
            commands::provider_config::set_active_provider,
            commands::ai::list_providers,
            commands::ai::save_provider_credentials,
            commands::ai::delete_provider_credentials,
            commands::ai::has_provider_credentials,
            commands::ai::test_provider_connection,
            commands::ai::decompose,
            commands::ai::transcribe_audio,
            commands::conversation::create_conversation,
            commands::conversation::list_conversations,
            commands::conversation::get_conversation,
            commands::conversation::delete_conversation,
            commands::conversation::rename_conversation,
            commands::chat::chat,
            commands::window::show_main,
            commands::window::toggle_quick,
            commands::window::hide_quick,
            commands::window::open_settings,
            commands::shortcut::get_summon_shortcut,
            commands::shortcut::set_summon_shortcut,
            commands::shortcut::reset_summon_shortcut,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(desktop)]
fn register_global_shortcuts(app: &tauri::AppHandle, shortcut_str: &str) -> anyhow::Result<()> {
    let summon = Shortcut::from_str(shortcut_str)
        .or_else(|_| Shortcut::from_str(commands::shortcut::DEFAULT_SHORTCUT))?;
    app.global_shortcut().register(summon)?;
    Ok(())
}

fn build_tray(app: &tauri::AppHandle) -> anyhow::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show Acorn", true, None::<&str>)?;
    let summon = MenuItem::with_id(app, "summon", "Summon  ⌘⇧A", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Acorn", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &summon, &settings, &separator, &quit])?;

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
