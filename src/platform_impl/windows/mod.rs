//! Windows-specific implementation for the observer.

pub mod error;
pub mod event;
pub mod observer;
mod task;

pub use error::PlatformError;
pub use observer::PlatformWindowObserver;

pub use window_getter::platform_impl::PlatformWindow;
