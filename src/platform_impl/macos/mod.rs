//! Bindings for macOS

pub mod application_observer;
pub mod ax_function;
pub mod ax_observer;
pub mod error_helper;
pub mod event_helper;
mod event_loop;
pub mod window;
mod window_observer;

pub use accessibility_sys::AXError as OSError;
pub use window_observer::MacOSWindowObserver;
