#![doc = include_str!("../README.md")]

pub use window_getter;

pub mod platform_impl;
pub mod window;

use crate::platform_impl::PlatformWindowObserver;

pub use ::tokio;
pub use window::Window;

/// Represents errors that can occur in the library.
#[non_exhaustive]
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

/// Represents a filter for window events.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventFilter {
    /// Whether to observe [`Event::Foregrounded`] events.
    pub foregrounded: bool,
    /// Whether to observe [`Event::Backgrounded`] events.
    pub backgrounded: bool,
    /// Whether to observe [`Event::Focused`] events.
    pub focused: bool,
    /// Whether to observe [`Event::Unfocused`] events.
    pub unfocused: bool,
    /// Whether to observe [`Event::Created`] events.
    pub created: bool,
    /// Whether to observe [`Event::Resized`] events.
    pub resized: bool,
    /// Whether to observe [`Event::Moved`] events.
    pub moved: bool,
    /// Whether to observe [`Event::Closed`] events.
    pub closed: bool,
}

impl EventFilter {
    /// Creates a new `EventFilter` with all events enabled.
    pub fn all() -> Self {
        Self {
            foregrounded: true,
            backgrounded: true,
            focused: true,
            unfocused: true,
            created: true,
            resized: true,
            moved: true,
            closed: true,
        }
    }

    /// Creates a new `EventFilter` with no events enabled.
    pub fn empty() -> Self {
        Self {
            foregrounded: false,
            backgrounded: false,
            focused: false,
            unfocused: false,
            created: false,
            resized: false,
            moved: false,
            closed: false,
        }
    }

    pub(crate) fn should_dispatch(&self, event: &Event) -> bool {
        matches!(event, Event::Foregrounded { .. }) && self.foregrounded
            || matches!(event, Event::Backgrounded { .. }) && self.backgrounded
            || matches!(event, Event::Focused { .. }) && self.focused
            || matches!(event, Event::Unfocused { .. }) && self.unfocused
            || matches!(event, Event::Created { .. }) && self.created
            || matches!(event, Event::Resized { .. }) && self.resized
            || matches!(event, Event::Moved { .. }) && self.moved
            || matches!(event, Event::Closed { .. }) && self.closed
    }
}

/// Represents events that can be observed on a window.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Event {
    /// The window was created.
    Created { window: Window },
    /// The window was resized.
    Resized { window: Window },
    /// The window was moved.
    Moved { window: Window },
    /// The window was brought to the foreground.
    /// This event does not mean the window has gained input focus.
    Foregrounded { window: Window },
    /// The window was backgrounded. It is opposite of [`Event::Foregrounded`].
    Backgrounded { window: Window },
    /// The windows was focused.
    Focused { window: Window },
    /// The window was unfocused.
    Unfocused { window: Window },
    /// The window was closed.
    Closed { window_id: window_getter::WindowId },
}

/// A type alias for the result of an event.
/// `Err` means that the event could not be processed, and `Ok` contains the event.
pub type EventResult = Result<Event, platform_impl::PlatformError>;
/// A type alias for the window event transmission channel.
pub type EventTx = tokio::sync::mpsc::UnboundedSender<EventResult>;

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
