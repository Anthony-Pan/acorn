mod commands;
mod db;
mod error;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ));

    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_global_shortcut::Builder::new().build());

    builder
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let db_path = app_data_dir.join("acorn.db");

            let database = tauri::async_runtime::block_on(async {
                db::Database::initialize(&db_path).await
            })?;

            app.manage(database);
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
