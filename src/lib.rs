pub mod platform_impl;
pub mod window;

pub use window::Window;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("The event listener is already started.")]
    AlreadyStarted,
    #[error("The event listener is already stopped.")]
    AlreadyStopped,
    #[error("Permission denied.")]
    PermissinoDenied,
    #[error("A platform-specific error occurred: {0:?}")]
    PlatformSpecificError(#[source] platform_impl::OSError),
}

#[derive(PartialEq)]
pub enum Event {
    Resized,
    Moved,
    Activated,
}

pub type EventCallback = Box<dyn Fn(Event, Window) + Send + Sync>;

pub struct WindowObserver {
    sys: platform_impl::WindowObserver,
}

impl WindowObserver {
    pub fn new(pid: i32, callback: EventCallback) -> Result<Self, Error> {
        Ok(Self {
            sys: platform_impl::WindowObserver::new(pid, callback)?,
        })
    }

    pub fn add_target_event(&mut self, target: Event) -> Result<(), Error> {
        self.sys.add_target_event(target)
    }

    pub fn remove_target_event(&mut self, target: Event) -> Result<(), Error> {
        self.sys.remove_target_event(target)
    }

    pub fn start(&mut self) -> Result<(), Error> {
        self.sys.start()
    }

    pub fn stop(&mut self) -> Result<(), Error> {
        self.sys.stop()
    }

    pub fn join(&mut self) {
        self.sys.join();
    }

    pub fn run(&mut self) -> Result<(), Error> {
        self.start()?;
        self.sys.join();
        Ok(())
    }
}
