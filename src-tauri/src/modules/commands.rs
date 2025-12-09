use crate::modules::services::chat::ChatService;
use crate::modules::services::input::InputService;
use crate::modules::services::window::{PlatformWindowService, traits::WindowService};
use crate::modules::core::types::WindowRect;
use tauri::Manager;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub fn get_diablo_rect(app_handle: tauri::AppHandle) -> Option<WindowRect> {
    let service = PlatformWindowService;
    service.get_diablo_rect(&app_handle)
}

#[tauri::command]
pub fn press_key(sequence: String) -> Result<(), String> {
    InputService::press_key(sequence)
}

#[tauri::command]
pub fn is_diablo_focused() -> bool {
    let service = PlatformWindowService;
    service.is_diablo_focused()
}

#[tauri::command]
pub async fn open_project_diablo2_webview(app_handle: tauri::AppHandle) -> Result<(), String> {
    // Spawn a new thread to avoid deadlocks on Windows
    std::thread::spawn(move || {
        let _ = crate::modules::webview::open_project_diablo2_webview(app_handle);
    });
    Ok(())
}

#[tauri::command]
pub fn update_window_bounds(app_handle: tauri::AppHandle) -> Result<(), String> {
    let service = PlatformWindowService;
    if let Some(bounds) = service.get_appropriate_window_bounds(&app_handle) {
        if let Some(main_window) = app_handle.get_webview_window("main") {
            let _ = main_window.set_position(tauri::PhysicalPosition::new(bounds.x as f64, bounds.y as f64));
            let _ = main_window.set_size(tauri::PhysicalSize::new(bounds.width as f64, bounds.height as f64));
        }
    }
    Ok(())
}

#[tauri::command]
pub fn set_window_click_through(app_handle: tauri::AppHandle, ignore: bool) -> Result<(), String> {
    if let Some(main_window) = app_handle.get_webview_window("main") {
        main_window.set_ignore_cursor_events(ignore)
            .map_err(|e| format!("Failed to set click-through: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub fn force_window_focus(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(main_window) = app_handle.get_webview_window("main") {
        main_window.set_focus()
            .map_err(|e| format!("Failed to set window focus: {}", e))?;
        main_window.set_always_on_top(true)
            .map_err(|e| format!("Failed to set always on top: {}", e))?;
        // Reset always on top after a short delay
        let app_handle_clone = app_handle.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(100));
            if let Some(window) = app_handle_clone.get_webview_window("main") {
                let _ = window.set_always_on_top(false);
            }
        });
    }
    Ok(())
}

#[tauri::command]
pub fn reposition_toast_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    use tauri::{PhysicalPosition, PhysicalSize};

    let service = PlatformWindowService;
    // Get bounds of the focused area (Diablo or work area)
    let bounds = service.get_appropriate_window_bounds(&app_handle)
        .ok_or("Could not get window bounds")?;

    // Toast window size
    let toast_width = 400;
    let toast_height = 200;
    let margin = 10;

    let x = bounds.x + bounds.width - toast_width - margin;
    let y = bounds.y + bounds.height - toast_height - margin;

    if let Some(toast_window) = app_handle.get_webview_window("toast") {
        toast_window
            .set_position(PhysicalPosition::new(x as f64, y as f64))
            .map_err(|e| format!("Failed to set toast window position: {}", e))?;
        toast_window
            .set_size(PhysicalSize::new(toast_width as f64, toast_height as f64))
            .map_err(|e| format!("Failed to set toast window size: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub fn start_chat_watcher(app_handle: tauri::AppHandle, custom_d2_dir: Option<String>) -> Result<(), String> {
    ChatService::start_watching(app_handle, custom_d2_dir)
}

#[tauri::command]
pub fn stop_chat_watcher() -> Result<(), String> {
    ChatService::stop_watching()
}

#[tauri::command]
pub fn get_diablo2_directory(custom_path: Option<String>) -> Option<String> {
    ChatService::find_diablo2_directory(custom_path.as_deref())
        .and_then(|p| p.to_str().map(|s| s.to_string()))
}

#[tauri::command]
pub fn auto_detect_diablo2_directory() -> Option<String> {
    ChatService::find_diablo2_directory(None)
        .and_then(|p| p.to_str().map(|s| s.to_string()))
}
