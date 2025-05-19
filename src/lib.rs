pub mod platform_impl;
pub mod window;

pub use window::Window;

/// Represents errors that can occur in the library.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The event listener is already started.
    #[error("The event listener is already started.")]
    AlreadyStarted,
    /// The event listener is already stopped.
    #[error("The event listener is already stopped.")]
    AlreadyStopped,
    /// Permission denied error.
    #[error("Permission denied.")]
    PermissinoDenied,
    /// A platform-specific error occurred.
    #[error("A platform-specific error occurred: {0:?}")]
    PlatformSpecificError(platform_impl::OSError),
}

/// Represents events that can be observed on a window.
#[derive(Debug, PartialEq)]
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

/// Observes window events.
pub struct WindowObserver {
    sys: platform_impl::WindowObserver,
}

impl WindowObserver {
    /// Creates a new `WindowObserver` for a given process ID and event channel.
    /// It will observe events of the specified process application.
    pub fn new(pid: i32, event_tx: EventTx) -> Result<Self, Error> {
        Ok(Self {
            sys: platform_impl::WindowObserver::new(pid, event_tx)?,
        })
    }

    /// Adds a event type to be observed.
    pub fn add_target_event(&mut self, target: Event) -> Result<(), Error> {
        self.sys.add_target_event(target)
    }

    /// Removes a event from being observed.
    pub fn remove_target_event(&mut self, target: Event) -> Result<(), Error> {
        self.sys.remove_target_event(target)
    }

    /// Starts the observer.
    pub fn start(&mut self) -> Result<(), Error> {
        self.sys.start()
    }

    /// Stops the observer.
    pub fn stop(&mut self) -> Result<(), Error> {
        self.sys.stop()
    }

    /// Blocks the current thread.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** It must to be called when there are no `CFRunLoop`.
    ///   Applications using most of the GUI libraries have a `CFRunLoop` running, so this is not needed.
    pub fn join(&mut self) {
        self.sys.join();
    }

    /// Runs the observer, starting it and blocking until it finishes.
    /// It will call `start` and then `join`.
    ///
    /// If you want to run the observer with a GUI application, you should only call `start` without `run`.
    /// And call `stop` when the application is closed.
    pub fn run(&mut self) -> Result<(), Error> {
        self.start()?;
        self.sys.join();
        Ok(())
    }

    /// Retrieves the platform-specific observer.
    /// `WindowObserver` is a wrapper of `platform_impl::WindowObserver`.
    pub fn sys(&self) -> &platform_impl::WindowObserver {
        &self.sys
    }
}
