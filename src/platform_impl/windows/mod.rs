//! Windows-specific implementation for the observer.

pub mod error;
mod event_interpreter;
mod hook_task;
pub mod observer;

pub type PlatformWindow = window_getter::platform_impl::PlatformWindow;
pub type PlatformWindowObserver = observer::WindowsWindowObserver;
pub type PlatformError = error::WindowsError;
