//! macOS-specific implementation for the observer.

pub mod binding_ax_function;
pub mod binding_ax_observer;
pub mod error;
mod event_interpreter;
mod event_loop;
pub mod window;
pub mod window_observer;

pub type PlatformError = error::MacOSError;
pub type PlatformWindowObserver = window_observer::MacOSWindowObserver;
pub type PlatformWindow = window::WindowUIElement;
