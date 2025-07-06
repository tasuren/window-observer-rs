use window_getter::Bounds;

use crate::{platform_impl::PlatformWindow, Error};

/// Represents the size of a window.
#[derive(Default, Debug)]
pub struct Size {
    /// The width of the window.
    pub width: f64,
    /// The height of the window.
    pub height: f64,
}

/// Represents the position of a window.
#[derive(Default, Debug)]
pub struct Position {
    /// The x-coordinate of the window.
    pub x: f64,
    /// The y-coordinate of the window.
    pub y: f64,
}

impl From<Bounds> for Size {
    fn from(value: Bounds) -> Self {
        Size {
            width: value.width(),
            height: value.height(),
        }
    }
}

impl From<Bounds> for Position {
    fn from(value: Bounds) -> Self {
        Position {
            x: value.x(),
            y: value.y(),
        }
    }
}

/// A wrapper around platform-specific window implementations.
pub struct Window(pub(crate) PlatformWindow);

impl Window {
    /// Retrieves the underlying platform-specific window implementation.
    pub fn inner(&self) -> &PlatformWindow {
        &self.0
    }

    /// Retrieves the title of the window.
    ///
    /// # Platform-specific
    /// - **macOS:** It will always return [`Some`] when it is ok.
    pub fn title(&self) -> Result<Option<String>, Error> {
        #[cfg(target_os = "macos")]
        {
            Ok(Some(self.0.title()?))
        }
        #[cfg(target_os = "windows")]
        {
            self.0
                .title()
                .map_err(|e| Error::PlatformSpecificError(e.into()))
        }
    }

    /// Retrieves the size of the window.
    pub fn size(&self) -> Result<Size, Error> {
        #[cfg(target_os = "macos")]
        {
            Ok(self.0.size()?)
        }
        #[cfg(target_os = "windows")]
        {
            Ok(window_getter::Bounds::new(
                self.0
                    .visible_rect()
                    .map_err(|e| Error::PlatformSpecificError(e.into()))?,
            )
            .into())
        }
    }

    /// Retrieves the position of the window.
    pub fn position(&self) -> Result<Position, Error> {
        #[cfg(target_os = "macos")]
        {
            Ok(self.0.position()?)
        }
        #[cfg(target_os = "windows")]
        {
            Ok(window_getter::Bounds::new(
                self.0
                    .visible_rect()
                    .map_err(|e| Error::PlatformSpecificError(e.into()))?,
            )
            .into())
        }
    }

    /// Checks if the window is currently active.
    ///
    /// # Platform-specific
    /// - **windows:** It will always return [`Ok`].
    pub fn is_active(&self) -> Result<bool, Error> {
        #[cfg(target_os = "macos")]
        {
            Ok(self.0.is_active()?)
        }
        #[cfg(target_os = "windows")]
        {
            Ok(self.0.is_foreground())
        }
    }

    /// Retrieves the unique identifier of the window.
    ///
    /// # Platform-specific
    /// - **macOS:** It will return a `CGWindowID` which is a unique identifier for the window.
    ///   **Warning:** It uses the private API `_AXUIElementGetWindow` of macOS.
    /// - **windows:** It will always return [`Ok`].
    #[cfg(feature = "macos-private-apis")]
    pub fn id(&self) -> Result<window_getter::WindowId, Error> {
        #[cfg(target_os = "macos")]
        {
            Ok(window_getter::WindowId::new(self.0.id()?))
        }
        #[cfg(target_os = "windows")]
        {
            Ok(window_getter::WindowId::new(self.0.hwnd()))
        }
    }

    /// Retrieves the `Window` implementation by [window-getter-rs][window-getter-rs].
    ///
    /// [window-getter-rs]: https://github.com/tasuren/window-getter-rs
    #[cfg(feature = "macos-private-apis")]
    pub fn create_window_getter_window(&self) -> Result<Option<window_getter::Window>, Error> {
        #[cfg(target_os = "macos")]
        {
            Ok(window_getter::get_window(self.id()?).expect("No window environment found"))
        }
        #[cfg(target_os = "windows")]
        {
            let window =
                unsafe { window_getter::platform_impl::PlatformWindow::new(self.inner().hwnd()) };
            Ok(Some(window_getter::Window::new(window)))
        }
    }
}
