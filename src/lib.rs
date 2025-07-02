#![doc = include_str!("../README.md")]

pub use window_getter;

pub mod platform_impl;
pub mod window;

use crate::platform_impl::PlatformWindowObserver;

pub use window::Window;
pub use ::{smallvec, smallvec::smallvec, tokio};

/// Represents errors that can occur in the library.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The process ID is invalid for observing windows.
    ///
    /// # Platform-specific
    /// - **windows:** This occurs when the process ID is zero.
    /// - **macOS:** This does not occur on macOS.
    #[error("The process ID is invalid: {0}")]
    InvalidProcessId(u32),
    /// This occurs when the application is not ready yet.
    /// This also occurs when the application that has given PID is not found.
    ///
    /// # Platform-specific
    /// - **windows:** This does not occur on windows.
    #[error("Something went wrong")]
    SomethingWentWrong,
    /// The application does not support observing window events.
    ///
    /// # Platform-specific
    /// - **windows:** This does not occur on windows.
    #[error("The application does not support observing window")]
    NotSupported,
    /// Permission denied error. This error only occurs on macOS.
    #[error("Permission denied.")]
    PermissionDenied,
    /// A platform-specific error occurred.
    #[error("A platform-specific error occurred: {0:?}")]
    PlatformSpecificError(#[from] platform_impl::PlatformError),
}

/// Represents events that can be observed on a window.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Event {
    /// The window was resized.
    Resized,
    /// The window was moved.
    Moved,
    /// The window was activated.
    ///
    /// # Platform-specific
    /// - **windows:** This event is occurred when the window is foregrounded.
    Activated,
    /// The window was deactivated. It is opposite of [`Event::Activated`].
    Deactivated,
}

/// A type alias for the window event transmission channel.
pub type EventTx = tokio::sync::mpsc::UnboundedSender<(Window, Event)>;
/// A type alias for the event filter used to specify which events to observe.
pub type EventFilter = smallvec::SmallVec<[Event; 4]>;

/// Observes window events.
pub struct WindowObserver(PlatformWindowObserver);

impl WindowObserver {
    /// Creates a new [`WindowObserver`] for a given process ID and event channel
    /// and start the observer.
    pub async fn start(
        pid: u32,
        event_tx: EventTx,
        event_filter: EventFilter,
    ) -> Result<Self, Error> {
        #[cfg(target_os = "macos")]
        let pid = pid as i32;

        Ok(Self(
            PlatformWindowObserver::start(pid, event_tx, event_filter).await?,
        ))
    }

    /// Stops the observer and cleans up resources.
    ///
    /// # Notes
    /// If you don't call this method, the observer will continue to run until droped.
    ///
    /// # Platform-specific
    /// - **macOS:** It will always return [`Ok`].
    pub async fn stop(self) -> Result<(), Error> {
        #[cfg(target_os = "macos")]
        self.0.stop().await;
        #[cfg(target_os = "windows")]
        self.0.stop().await?;

        Ok(())
    }

    /// Returns underlying platform-specific observer.
    pub fn inner(&self) -> &PlatformWindowObserver {
        &self.0
    }
}
