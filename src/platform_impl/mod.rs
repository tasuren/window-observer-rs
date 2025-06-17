#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::{
    window::MacOSWindow as Window, MacOSWindowObserver as WindowObserver, PlatformError,
};

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::{
    PlatformError, PlatformWindow as Window, PlatformWindowObserver as WindowObserver,
};
