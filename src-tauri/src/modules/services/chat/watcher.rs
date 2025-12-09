use super::parser;
use std::fs;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use notify::{Watcher, RecommendedWatcher, RecursiveMode, Event, EventKind};

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

static WATCHER_HANDLE: Mutex<Option<Arc<Mutex<Option<RecommendedWatcher>>>>> = Mutex::new(None);
static LAST_POSITION: Mutex<u64> = Mutex::new(0);

pub struct ChatService;

impl ChatService {
    pub fn find_diablo2_directory(custom_path: Option<&str>) -> Option<PathBuf> {
        if let Some(custom) = custom_path {
            if !custom.is_empty() {
                let path = PathBuf::from(custom);
                if path.exists() {
                    return Some(path);
                }
            }
        }

        #[cfg(target_os = "windows")]
        if let Some(path) = Self::find_diablo2_in_registry() {
            if path.exists() {
                return Some(path);
            }
        }

        #[cfg(target_os = "windows")]
        let common_paths = vec![
            PathBuf::from(r"C:\Diablo II"),
            PathBuf::from(r"D:\Diablo II"),
            PathBuf::from(r"E:\Diablo II"),
            PathBuf::from(r"C:\Program Files\Diablo II"),
            PathBuf::from(r"C:\Program Files (x86)\Diablo II"),
            PathBuf::from(r"D:\Program Files\Diablo II"),
            PathBuf::from(r"D:\Program Files (x86)\Diablo II"),
        ];

        #[cfg(not(target_os = "windows"))]
        let common_paths = {
            let home = std::env::var("HOME").unwrap_or_default();
            let home_path = PathBuf::from(home);
            vec![
                home_path.join("Games/Diablo II"),
                home_path.join("Games/project-diablo-2"),
                home_path.join(".wine/drive_c/Program Files (x86)/Diablo II"),
            ]
        };

        for path in common_paths {
            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    #[cfg(target_os = "windows")]
    fn find_diablo2_in_registry() -> Option<PathBuf> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if let Ok(key) = hklm.open_subkey(r"SOFTWARE\Blizzard Entertainment\Diablo II") {
            if let Ok(install_path) = key.get_value::<String, _>("InstallPath") {
                return Some(PathBuf::from(install_path));
            }
        }

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(key) = hkcu.open_subkey(r"SOFTWARE\Blizzard Entertainment\Diablo II") {
            if let Ok(install_path) = key.get_value::<String, _>("InstallPath") {
                return Some(PathBuf::from(install_path));
            }
        }

        None
    }

    pub fn get_chat_log_path(custom_d2_dir: Option<&str>) -> Option<PathBuf> {
        let d2_dir = Self::find_diablo2_directory(custom_d2_dir)?;
        let logs_dir = d2_dir.join("ProjectD2").join("pd2logs");
        
        if let Err(_e) = fs::create_dir_all(&logs_dir) {
            return None;
        }

        let log_file = logs_dir.join("pd2_chat.log");
        
        if !log_file.exists() {
            if let Err(_e) = fs::File::create(&log_file) {
                return None;
            }
        }

        Some(log_file)
    }

    pub fn get_game_log_path(custom_d2_dir: Option<&str>) -> Option<PathBuf> {
        let d2_dir = Self::find_diablo2_directory(custom_d2_dir)?;
        let logs_dir = d2_dir.join("ProjectD2").join("pd2logs");
        
        if let Err(_e) = fs::create_dir_all(&logs_dir) {
            return None;
        }

        let log_file = logs_dir.join("pd2_game.log");
        
        if !log_file.exists() {
            if let Err(_e) = fs::File::create(&log_file) {
                return None;
            }
        }

        Some(log_file)
    }

    fn read_new_lines(file_path: &Path, app_handle: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
        let file_size = file_path.metadata().map(|m| m.len()).unwrap_or(0);
        
        let file = fs::File::open(file_path)?;
        let mut reader = BufReader::new(file);
        
        let mut last_pos = LAST_POSITION.lock().unwrap();
        
        if file_size < *last_pos {
            *last_pos = file_size;
            return Ok(());
        }
        
        if file_size == 0 {
            *last_pos = 0;
            return Ok(());
        }
        
        if *last_pos >= file_size {
            return Ok(());
        }
        
        if let Err(_e) = reader.seek(SeekFrom::Start(*last_pos)) {
            if let Ok(metadata) = file_path.metadata() {
                *last_pos = metadata.len();
                if let Err(_e2) = reader.seek(SeekFrom::Start(*last_pos)) {
                    return Ok(());
                }
            } else {
                return Ok(());
            }
        }
        
        let mut line = Vec::new();
        
        loop {
            match reader.read_until(b'\n', &mut line) {
                Ok(0) => break,
                Ok(_bytes_read) => {
                    let line_str = String::from_utf8_lossy(&line);
                    
                    if let Some(whisper) = parser::parse_whisper(&line_str) {
                        let _ = app_handle.emit("whisper-received", whisper.clone());
                    }
                    line.clear();
                }
                Err(_e) => {
                    if let Ok(current_pos) = reader.seek(SeekFrom::Current(0)) {
                        *last_pos = current_pos;
                    }
                    break;
                }
            }
        }
        
        match reader.seek(SeekFrom::Current(0)) {
            Ok(current_pos) => {
                *last_pos = current_pos;
            }
            Err(_e) => {
                if let Ok(metadata) = file_path.metadata() {
                    *last_pos = metadata.len();
                }
            }
        }
        
        Ok(())
    }

    pub fn start_watching(app_handle: AppHandle, custom_d2_dir: Option<String>) -> Result<(), String> {
        let log_path = match Self::get_chat_log_path(custom_d2_dir.as_deref()) {
            Some(path) => path,
            None => {
                let msg = "Could not find or create chat log file. Please check your Diablo II Directory settings.";
                let _ = app_handle.emit("error", msg);
                return Err(msg.to_string());
            }
        };
        
        let _ = Self::get_game_log_path(custom_d2_dir.as_deref());
        
        if let Ok(file) = fs::File::open(&log_path) {
            if let Ok(metadata) = file.metadata() {
                *LAST_POSITION.lock().unwrap() = metadata.len();
            }
        }

        let log_path_for_watcher = log_path.clone();
        
        let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |result: Result<Event, notify::Error>| {
            match result {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Modify(_)) {
                        let app_handle_clone = app_handle.clone();
                        let log_path_clone = log_path_for_watcher.clone();
                        
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        
                        let _ = Self::read_new_lines(&log_path_clone, app_handle_clone);
                    }
                }
                Err(_e) => {}
            }
        }).map_err(|e| format!("Failed to create file watcher: {}", e))?;

        watcher.watch(&log_path, RecursiveMode::NonRecursive)
            .map_err(|e| format!("Failed to watch chat log file: {}", e))?;

        *WATCHER_HANDLE.lock().unwrap() = Some(Arc::new(Mutex::new(Some(watcher))));

        Ok(())
    }

    pub fn stop_watching() -> Result<(), String> {
        let mut handle_guard = WATCHER_HANDLE.lock().unwrap();
        if let Some(arc_watcher) = handle_guard.take() {
            if let Ok(mut watcher_guard) = arc_watcher.lock() {
                watcher_guard.take();
            }
        }
        Ok(())
    }
}
