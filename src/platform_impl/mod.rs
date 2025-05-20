#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::{
    window::MacOSWindow as Window, AXError as OSError, MacOSWindowObserver as WindowObserver,
};

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::{OSError, Window, WindowObserver};
