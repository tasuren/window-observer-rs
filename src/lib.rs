pub mod platform_impl;
pub mod window;

pub use window::Window;
pub use ::{smallvec, smallvec::smallvec};

/// Represents errors that can occur in the library.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Permission denied error.
    #[error("Permission denied.")]
    PermissionDenied,
    /// A platform-specific error occurred.
    #[error("A platform-specific error occurred: {0:?}")]
    PlatformSpecificError(#[from] platform_impl::OSError),
}

/// Represents events that can be observed on a window.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// The window was resized.
    Resized,
    /// The window was moved.
    Moved,
    /// The window was activated.
    Activated,
}

/// A type alias for the window event transmission channel.
pub type EventTx = tokio::sync::mpsc::UnboundedSender<(crate::Window, Event)>;
/// A type alias for the event filter used to specify which events to observe.
pub type EventFilter = smallvec::SmallVec<[Event; 3]>;

/// Observes window events.
pub struct WindowObserver {
    sys: platform_impl::WindowObserver,
}

impl WindowObserver {
    /// Creates a new `WindowObserver` for a given process ID and event channel
    /// and start the observer.
    pub async fn start(
        pid: i32,
        event_tx: EventTx,
        event_filter: EventFilter,
    ) -> Result<Self, Error> {
        Ok(Self {
            sys: platform_impl::WindowObserver::start(pid, event_tx, event_filter).await?,
        })
    }

    /// Stops the observer and cleans up resources.
    pub async fn stop(self) -> Result<(), Error> {
        #[cfg(target_os = "macos")]
        self.sys.stop().await;
        #[cfg(target_os = "windows")]
        self.sys.stop().await?;

        Ok(())
    }
}
