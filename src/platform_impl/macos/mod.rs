//! Bindings for macOS
pub mod ax_observer;
pub mod error_helper;
pub mod event_helper;
pub mod ax_function;
mod observer;
mod thread;
pub mod window;

pub use accessibility_sys::AXError;
pub use observer::MacOSWindowObserver;
