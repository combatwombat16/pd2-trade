use crate::modules::core::types::WindowRect;
use tauri::AppHandle;

pub trait WindowService {
    fn get_diablo_rect(&self, app: &AppHandle) -> Option<WindowRect>;
    fn is_diablo_focused(&self) -> bool;
    fn get_appropriate_window_bounds(&self, app: &AppHandle) -> Option<WindowRect>;
    fn initialize_foreground_monitoring<F: Fn() + Send + 'static>(&self, callback: F);
    fn cleanup_foreground_monitoring(&self);
}
