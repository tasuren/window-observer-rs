#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::{
    error::MacOSError as PlatformError,
    window_observer::MacOSWindowObserver as PlatformWindowObserver,
    window::WindowUIElement as PlatformWindow
};

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::{PlatformError, PlatformWindow, PlatformWindowObserver};
