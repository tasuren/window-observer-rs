#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::{window::MacOSWindow as Window, MacOSWindowObserver as WindowObserver, OSError};

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::{OSError, WindowsWindow as Window, WindowsWindowObserver as WindowObserver};
