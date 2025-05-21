//! Windows-specific implementation for the observer.

pub mod error;
pub mod event;
pub mod observer;
mod task;
pub mod window;

pub use error::OSError;
pub use observer::WindowsWindowObserver;
pub use window::WindowsWindow;
