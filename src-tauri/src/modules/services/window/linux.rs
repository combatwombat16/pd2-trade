use super::traits::WindowService;
use crate::modules::core::types::WindowRect;
use serde::Deserialize;
use std::fs;
use tauri::{AppHandle, Emitter, Manager};

pub struct LinuxWindowService;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppConfig {
    pd2_install_dir: Option<String>,
}

impl LinuxWindowService {
    fn get_work_area(app: &AppHandle) -> Option<WindowRect> {
        let config_dir = app.path().app_config_dir().ok()?;
        let config_path = config_dir.join("settings.json");

        if !config_path.exists() {
            println!("Config file not found at {:?}", config_path);
            return None;
        }

        let config_content = fs::read_to_string(&config_path).ok()?;
        let config: AppConfig = serde_json::from_str(&config_content).ok()?;

        let install_dir = config.pd2_install_dir?;
        let d2gl_path = std::path::Path::new(&install_dir).join("d2gl.json");

        if !d2gl_path.exists() {
            let _ = app.emit(
                "error",
                format!(
                    "d2gl.json not found at {:?}. Overlay positioning may be incorrect.",
                    d2gl_path
                ),
            );
            return None;
        }

        let contents = fs::read_to_string(&d2gl_path).ok()?;

        let json: serde_json::Value = serde_json::from_str(&contents).ok()?;
        let width = json["screen"]["window_size_width"].as_i64()? as f64;
        let height = json["screen"]["window_size_height"].as_i64()? as f64;
        let mut x = 0.0;
        let mut y = 0.0;
        if !json["screen"]["window_centered"].as_bool()? {
            x = json["screen"]["window_position_x"].as_f64()?;
            y = json["screen"]["window_position_y"].as_f64()?;
        }
        Some(WindowRect {
            x: x as i32,
            y: y as i32,
            width: width as i32,
            height: height as i32,
        })
    }
}

impl WindowService for LinuxWindowService {
    fn get_diablo_rect(&self, app: &AppHandle) -> Option<WindowRect> {
        Self::get_work_area(app)
    }

    fn is_diablo_focused(&self) -> bool {
        true
    }

    fn get_appropriate_window_bounds(&self, app: &AppHandle) -> Option<WindowRect> {
        // On Linux, we always assume focused for now (simplification from original code)
        // or we can check logic. Original code: if is_diablo_focused() { get_diablo_rect } else { get_work_area }
        // But is_diablo_focused returns true. So it always calls get_diablo_rect -> get_work_area.
        Self::get_work_area(app)
    }

    fn initialize_foreground_monitoring<F: Fn() + Send + 'static>(&self, _callback: F) {
        // No-op on Linux
    }

    fn cleanup_foreground_monitoring(&self) {
        // No-op on Linux
    }
}
