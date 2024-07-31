pub mod platform_impl;
pub mod window;

pub use window::Window;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Permission denied.")]
    PermissinoDenied,
    #[error("A platform-specific error occurred: {0:?}")]
    PlatformSpecificError(#[source] platform_impl::OSError),
}

pub enum Event {
    Resized,
    Moved,
    Activated,
}

pub struct WindowObserver {
    sys: platform_impl::WindowObserver,
}

impl WindowObserver {
    pub fn new(pid: i32, callback: Box<dyn Fn(Event, Window)>) -> Result<Self, Error> {
        Ok(Self {
            sys: platform_impl::WindowObserver::new(pid, callback)?,
        })
    }

    pub fn add_target_event(&self, target: Event) {
        self.sys.add_target_event(target);
    }

    pub fn start(&mut self) -> Result<(), Error> {
        Ok(self.sys.start()?)
    }

    pub fn stop(&self) -> Result<(), ()> {
        Ok(self.sys.stop())
    }

    pub fn join(&self) {
        self.sys.join();
    }

    pub fn run(&mut self) -> Result<(), Error> {
        self.start()?;
        self.sys.join();
        Ok(())
    }
}
