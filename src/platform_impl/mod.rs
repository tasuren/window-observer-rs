#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::{
    ax_error::AXError as OSError, window::MacOSWindow as Window,
    MacOSWindowObserver as WindowObserver,
};

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::{OSError, Window, WindowObserver};
