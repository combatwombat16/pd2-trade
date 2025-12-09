pub mod traits;
#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "windows")]
pub use windows::WindowsWindowService as PlatformWindowService;

#[cfg(target_os = "linux")]
pub use linux::LinuxWindowService as PlatformWindowService;
