use crate::modules::commands;
use crate::modules::services::window::PlatformWindowService;
use crate::modules::services::window::traits::WindowService;
use tauri::{App, Manager, WebviewUrl, WebviewWindowBuilder};

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::RECT;
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, SPI_GETWORKAREA};

pub fn init(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let _handle = app.app_handle();
    let window_service = PlatformWindowService;

    #[cfg(target_os = "windows")]
    let (x, y, width, height) = {
        // Get appropriate bounds based on Diablo focus state
        if let Some(rect) = window_service.get_appropriate_window_bounds(app.app_handle()) {
            (
                rect.x as f64,
                rect.y as f64,
                rect.width as f64,
                rect.height as f64,
            )
        } else {
            // Fallback to work area
            let mut work_area = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            unsafe {
                SystemParametersInfoW(
                    SPI_GETWORKAREA,
                    0,
                    &mut work_area as *mut _ as *mut _,
                    0,
                );
            }
            let width = (work_area.right - work_area.left) as f64;
            let height = (work_area.bottom - work_area.top) as f64;
            let x = work_area.left as f64;
            let y = work_area.top as f64;
            (x, y, width, height)
        }
    };

    #[cfg(not(target_os = "windows"))]
    let (x, y, width, height) = {
        if let Some(rect) = window_service.get_appropriate_window_bounds(app.app_handle()) {
            (
                rect.x as f64,
                rect.y as f64,
                rect.width as f64,
                rect.height as f64,
            )
        } else {
            let monitor = app.primary_monitor().unwrap();
            if let Some(monitor) = monitor {
                let size = monitor.size();
                let position = monitor.position();
                (
                    position.x as f64,
                    position.y as f64,
                    size.width as f64,
                    size.height as f64,
                )
            } else {
                println!("Warning: No primary monitor detected. using default bounds.");
                (0.0, 0.0, 1920.0, 1080.0)
            }
        }
    };

    let win_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
        .title("PD2 Trader")
        .inner_size(width, height)
        .position(x, y)
        .decorations(false)
        .transparent(true)
        .visible(true)
        .focused(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true);

    let main_window = win_builder.build().unwrap();
    let _ = main_window.set_ignore_cursor_events(true);
    
    // Create toast window
    let _toast_window = WebviewWindowBuilder::new(app, "toast", WebviewUrl::App("toast".into()))
        .title("PD2 Trader - Toast")
        .inner_size(400.0, 200.0)
        .decorations(false)
        .transparent(true)
        .visible(false)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focusable(false)
        .build()
        .unwrap();
    
    // Initialize event-driven foreground monitoring
    let app_handle = app.app_handle().clone();
    let _ = commands::reposition_toast_window(app_handle.clone());
    window_service.initialize_foreground_monitoring(move || {
        let _ = commands::update_window_bounds(app_handle.clone());
        let _ = commands::reposition_toast_window(app_handle.clone());
    });

    #[cfg(debug_assertions)]
    main_window.open_devtools();
    Ok(())
}
