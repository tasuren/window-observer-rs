//! Bindings for macOS
pub mod ax_function;
pub mod ax_observer;
pub mod error_helper;
pub mod event_helper;
mod observer;
mod thread;
pub mod window;

pub use accessibility_sys::AXError as OSError;
pub use observer::MacOSWindowObserver;
