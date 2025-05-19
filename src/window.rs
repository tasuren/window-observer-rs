use crate::Error;

/// Represents the size of a window.
#[derive(Default, Debug)]
pub struct Size {
    /// The width of the window.
    pub width: i32,
    /// The height of the window.
    pub height: i32,
}

/// Represents the position of a window.
#[derive(Default, Debug)]
pub struct Position {
    /// The x-coordinate of the window.
    pub x: i32,
    /// The y-coordinate of the window.
    pub y: i32,
}

/// A wrapper around platform-specific window implementations.
pub struct Window(pub(crate) crate::platform_impl::Window);

impl Window {
    /// Retrieves the size of the window.
    pub fn get_size(&self) -> Result<Size, Error> {
        self.0.get_size()
    }

    /// Retrieves the position of the window.
    pub fn get_position(&self) -> Result<Position, Error> {
        self.0.get_position()
    }

    /// Checks if the window is currently active.
    pub fn is_active(&self) -> Result<bool, Error> {
        self.0.is_active()
    }
}
