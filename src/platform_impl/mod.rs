#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "macos")]
pub use macos::{Error as PlatformSpecificError, WindowObserver};
