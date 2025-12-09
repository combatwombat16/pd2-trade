use super::traits::WindowService;
use crate::modules::core::types::WindowRect;
use std::sync::Mutex;
use std::{ffi::OsStr, iter, os::windows::prelude::OsStrExt, ptr};
use tauri::AppHandle;
use windows_sys::Win32::{
    Foundation::{HWND, RECT},
    UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
    UI::WindowsAndMessaging::{
        FindWindowW, GetForegroundWindow, GetWindowRect, SystemParametersInfoW, SPI_GETWORKAREA,
        EVENT_SYSTEM_FOREGROUND, WINEVENT_OUTOFCONTEXT,
    },
};

static mut FOREGROUND_HOOK: Option<HWINEVENTHOOK> = None;
static CALLBACK: Mutex<Option<Box<dyn Fn() + Send>>> = Mutex::new(None);

unsafe extern "system" fn win_event_proc(
    _h_win_event_hook: HWINEVENTHOOK,
    _event: u32,
    _hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _dw_event_thread: u32,
    _dwms_event_time: u32,
) {
    if let Some(cb) = &*CALLBACK.lock().unwrap() {
        cb();
    }
}

pub struct WindowsWindowService;

impl WindowsWindowService {
    fn to_wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(iter::once(0)).collect()
    }

    fn get_work_area() -> Option<WindowRect> {
        let mut work_area = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let ok = unsafe {
            SystemParametersInfoW(
                SPI_GETWORKAREA,
                0,
                &mut work_area as *mut _ as *mut _,
                0,
            )
        };
        if ok == 0 {
            return None;
        }
        Some(WindowRect {
            x: work_area.left,
            y: work_area.top,
            width: work_area.right - work_area.left,
            height: work_area.bottom - work_area.top,
        })
    }
}

impl WindowService for WindowsWindowService {
    fn get_diablo_rect(&self, _app: &AppHandle) -> Option<WindowRect> {
        let title_w = Self::to_wide("Diablo II");
        let hwnd: HWND = unsafe { FindWindowW(ptr::null(), title_w.as_ptr()) };
        if hwnd == 0 {
            return None;
        }
        let mut r = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let ok = unsafe { GetWindowRect(hwnd, &mut r as *mut RECT) };
        if ok == 0 {
            return None;
        }
        Some(WindowRect {
            x: r.left,
            y: r.top,
            width: r.right - r.left,
            height: r.bottom - r.top,
        })
    }

    fn is_diablo_focused(&self) -> bool {
        let title_w = Self::to_wide("Diablo II");
        let hwnd: HWND = unsafe { FindWindowW(ptr::null(), title_w.as_ptr()) };
        if hwnd == 0 {
            return false;
        }
        let foreground = unsafe { GetForegroundWindow() };
        hwnd == foreground
    }

    fn get_appropriate_window_bounds(&self, app: &AppHandle) -> Option<WindowRect> {
        if self.is_diablo_focused() {
            self.get_diablo_rect(app)
        } else {
            Self::get_work_area()
        }
    }

    fn initialize_foreground_monitoring<F: Fn() + Send + 'static>(&self, callback: F) {
        unsafe {
            let hook = SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                0,
                Some(win_event_proc),
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            );
            FOREGROUND_HOOK = Some(hook);
            *CALLBACK.lock().unwrap() = Some(Box::new(callback));
        }
    }

    fn cleanup_foreground_monitoring(&self) {
        unsafe {
            if let Some(hook) = FOREGROUND_HOOK {
                UnhookWinEvent(hook);
                FOREGROUND_HOOK = None;
            }
            *CALLBACK.lock().unwrap() = None;
        }
    }
}
