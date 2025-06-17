#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::{window::PlatformWindow, PlatformError, PlatformWindowObserver};

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::{PlatformError, PlatformWindow, PlatformWindowObserver};
