#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::{
    error::MacOSError as PlatformError, window::WindowUIElement as PlatformWindow,
    window_observer::MacOSWindowObserver as PlatformWindowObserver,
};

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use window_getter::platform_impl::windows::WindowsWindow as PlatformWindow;
#[cfg(target_os = "windows")]
pub use windows::{
    error::WindowsError as PlatformError, observer::WindowsWindowObserver as PlatformWindowObserver,
};
