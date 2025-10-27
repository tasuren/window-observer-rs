use window_getter::Bounds;

use crate::{platform_impl::PlatformWindow, Error};

/// A wrapper around platform-specific window implementations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window(pub(crate) PlatformWindow);

impl Window {
    /// Creates a new `Window` instance from a platform-specific window.
    pub fn new(platform_window: PlatformWindow) -> Self {
        Self(platform_window)
    }

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
            Ok(self
                .0
                .visible_bounds()
                .map_err(|e| Error::PlatformSpecificError(e.into()))?
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
            Ok(self
                .0
                .visible_bounds()
                .map_err(|e| Error::PlatformSpecificError(e.into()))?
                .into())
        }
    }

    /// Checks if the window is currently focused.
    ///
    /// # Platform-specific
    /// - **Windows:** It will always return [`Ok`].
    pub fn is_focused(&self) -> Result<bool, Error> {
        #[cfg(target_os = "macos")]
        {
            Ok(self.0.is_focused()?)
        }
        #[cfg(target_os = "windows")]
        {
            Ok(self.0.is_foreground())
        }
    }

    /// Retrieves the unique identifier of the window.
    ///
    /// # Platform-specific
    /// - **macOS:** It will return a [`CGWindowID`][CGWindowID] which is wrapped by [`window_getter::WindowId`].
    ///   **Warning:** It uses the private API `_AXUIElementGetWindow` of macOS.
    /// - **Windows:** It will always return [`Ok`].
    ///
    /// [CGWindowID]: https://developer.apple.com/documentation/coregraphics/cgwindowid?language=objc
    #[cfg(feature = "macos-private-api")]
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
    /// # Panics
    /// On macOS, if there is no window environment, it will panic.
    ///
    /// [window-getter-rs]: https://github.com/tasuren/window-getter-rs
    ///
    /// # Platform-specific
    /// - **Windows:** It will always return `Ok(Some(Window))`.
    #[cfg(feature = "macos-private-api")]
    pub fn create_window_getter_window(&self) -> Result<Option<window_getter::Window>, Error> {
        #[cfg(target_os = "macos")]
        {
            Ok(window_getter::get_window(self.id()?).expect("No window environment found"))
        }
        #[cfg(target_os = "windows")]
        {
            let window = window_getter::platform_impl::PlatformWindow::new(self.inner().hwnd());
            Ok(Some(window_getter::Window::new(window)))
        }
    }
}

/// Represents the size of a window.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Size {
    /// The width of the window.
    pub width: f64,
    /// The height of the window.
    pub height: f64,
}

/// Represents the position of a window.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Position {
    /// The x-coordinate of the window.
    pub x: f64,
    /// The y-coordinate of the window.
    pub y: f64,
}

impl From<Bounds> for Size {
    fn from(value: Bounds) -> Self {
        Size {
            width: value.width,
            height: value.height,
        }
    }
}

impl From<Bounds> for Position {
    fn from(value: Bounds) -> Self {
        Position {
            x: value.x,
            y: value.y,
        }
    }
}
