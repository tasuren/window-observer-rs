#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::{PlatformError, PlatformWindow, PlatformWindowObserver};

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::{PlatformError, PlatformWindow, PlatformWindowObserver};
