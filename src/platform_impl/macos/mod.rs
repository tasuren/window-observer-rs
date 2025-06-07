//! macOS-specific implementation for the observer.

pub mod ax_function;
pub mod ax_observer;
pub mod error;
pub mod event;
pub mod event_loop;
pub mod window;
pub mod window_observer;

pub use error::OSError;
pub use window_observer::MacOSWindowObserver;
