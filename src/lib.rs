#![doc = include_str!("../README.md")]

pub use window_getter;

pub mod platform_impl;
pub mod window;

pub use ::tokio;
pub use window::{Position, Size, Window};

use crate::platform_impl::PlatformWindowObserver;

/// Represents errors that can occur in the library.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The process ID is invalid for observing windows.
    ///
    /// # Platform-specific
    /// - **Windows:** This occurs when the process ID is zero.
    /// - **macOS:** This does not occur on macOS.
    #[error("The process ID is invalid: {0}")]
    InvalidProcessId(u32),
    /// This occurs when the application is not ready yet.
    /// This also occurs when the application that has given PID is not found.
    ///
    /// # Platform-specific
    /// - **Windows:** This does not occur on windows.
    #[error("Something went wrong")]
    SomethingWentWrong,
    /// The application does not support observing window events.
    ///
    /// # Platform-specific
    /// - **Windows:** This does not occur on windows.
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
    /// Whether to observe [`Event::Hidden`] events.
    pub hidden: bool,
    /// Whether to observe [`Event::Showed`] events.
    pub showed: bool,
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
            hidden: true,
            showed: true,
            closed: true,
        }
    }

    /// Creates a new `EventFilter` with no events enabled.
    pub fn empty() -> Self {
        Default::default()
    }

    pub(crate) fn should_dispatch(&self, event: &Event) -> bool {
        matches!(event, Event::Foregrounded) && self.foregrounded
            || matches!(event, Event::Backgrounded) && self.backgrounded
            || matches!(event, Event::Focused) && self.focused
            || matches!(event, Event::Unfocused) && self.unfocused
            || matches!(event, Event::Created) && self.created
            || matches!(event, Event::Resized) && self.resized
            || matches!(event, Event::Moved) && self.moved
            || matches!(event, Event::Hidden) && self.hidden
            || matches!(event, Event::Showed) && self.showed
            || matches!(event, Event::Closed { .. }) && self.closed
    }
}

/// Represents events that can be observed on a window.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// The window was created.
    Created,
    /// The window was resized.
    Resized,
    /// The window was moved.
    Moved,
    /// The window was brought to the foreground.
    /// This event does not mean the window has gained input focus.
    Foregrounded,
    /// The window was backgrounded. It is opposite of [`Event::Foregrounded`].
    Backgrounded,
    /// The window was focused.
    ///
    /// # Platform-specific
    /// - **Windows:** This event is same as [`Event::Foregrounded`].
    ///   So this event and `Foregrounded` event are always dispatched together.
    /// - **macOS:** On macOS, a window does not lose focus even when miniaturized.
    ///   Therefore, this event will not be dispatched when the window is deminiaturized.
    Focused,
    /// The window was unfocused.
    ///
    /// # Platform-specific
    /// - **Windows:** This event is same as [`Event::Backgrounded`].
    ///   So this event and `Backgrounded` event are always dispatched together.
    /// - **macOS:** On macOS, a window does not lose focus even when miniaturized.
    ///   Therefore, this event will not be dispatched when the window is miniaturized
    Unfocused,
    /// The window was hidden.
    Hidden,
    /// The window was showed.
    Showed,
    /// The window was closed.
    Closed { window_id: window_getter::WindowId },
}

/// Represents a window that may or may not be available.
#[derive(Debug, Clone, PartialEq)]
pub enum MaybeWindowAvailable {
    /// The window is available.
    Available { window: Window, event: Event },
    /// The window is not available.
    /// This can happen when the window is closed.
    NotAvailable { event: Event },
}

/// A type alias for the result of an event.
/// `Err` means that the event could not be processed, and `Ok` contains the event.
pub type EventResult = Result<MaybeWindowAvailable, platform_impl::PlatformError>;
/// A type alias for the window event transmission channel.
pub type EventTx = tokio::sync::mpsc::UnboundedSender<EventResult>;
/// A type alias for the window event reception channel.
pub type EventRx = tokio::sync::mpsc::UnboundedReceiver<EventResult>;

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
