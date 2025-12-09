use tauri::Manager;

pub mod modules;
pub mod setup;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "windows")]
    if !modules::services::window::windows::is_elevated() {
        modules::services::window::windows::restart_as_admin();
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_app_exit::init())
        .setup(setup::init)
        .invoke_handler(tauri::generate_handler![
            modules::commands::get_diablo_rect,
            modules::commands::press_key,
            modules::commands::is_diablo_focused,
            modules::commands::open_project_diablo2_webview,
            modules::commands::update_window_bounds,
            modules::commands::set_window_click_through,
            modules::commands::force_window_focus,
            modules::commands::reposition_toast_window,
            modules::commands::start_chat_watcher,
            modules::commands::stop_chat_watcher,
            modules::commands::get_diablo2_directory,
            modules::commands::auto_detect_diablo2_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
