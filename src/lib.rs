pub mod platform_impl;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("A platform-specific error occurred: {0:?}")]
    PlatformSpecificError(#[from] platform_impl::PlatformSpecificError),
}

pub struct WindowObserver {
    sys: platform_impl::WindowObserver,
}

impl WindowObserver {
    pub fn new(pid: i32, callback: Box<dyn FnMut()>) -> Result<Self, Error> {
        Ok(Self {
            sys: platform_impl::WindowObserver::new(pid, callback)?,
        })
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
