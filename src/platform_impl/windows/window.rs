use binding::get_window_text;
use windows::Win32::{Foundation, UI::WindowsAndMessaging};

use super::OSError;
use crate::window;

mod binding {
    use std::mem::MaybeUninit;

    use windows::Win32::{Foundation, UI::WindowsAndMessaging};

    pub fn get_window_rect(hwnd: Foundation::HWND) -> windows::core::Result<Foundation::RECT> {
        let mut value = MaybeUninit::uninit();

        unsafe {
            WindowsAndMessaging::GetWindowRect(hwnd, value.as_mut_ptr())?;
            Ok(value.assume_init())
        }
    }

    pub fn get_window_text(hwnd: Foundation::HWND) -> windows::core::Result<String> {
        let mut buffer = [0u16; 256];
        let length = unsafe { WindowsAndMessaging::GetWindowTextW(hwnd, &mut buffer) };

        if length == 0 {
            return Err(windows::core::Error::from_win32());
        }

        let text = String::from_utf16_lossy(&buffer[..length as usize]);
        Ok(text)
    }
}

/// Represents a window on the Windows platform.
pub struct WindowsWindow(Foundation::HWND);
unsafe impl Send for WindowsWindow {}
unsafe impl Sync for WindowsWindow {}

impl WindowsWindow {
    /// Creates a new `WindowsWindow` from a window handle.
    pub fn new(hwnd: Foundation::HWND) -> Self {
        Self(hwnd)
    }

    /// Retrieves the window handle.
    pub fn hwnd(&self) -> Foundation::HWND {
        self.0
    }

    /// Retrieves the title of the window.
    pub fn get_title(&self) -> Result<String, OSError> {
        Ok(get_window_text(self.0)?)
    }

    /// Retrieves the rectangle of the window.
    pub fn get_rect(&self) -> Result<Foundation::RECT, OSError> {
        Ok(binding::get_window_rect(self.0)?)
    }

    /// Retrieves the size of the window.
    pub fn get_size(&self) -> Result<window::Size, OSError> {
        let rect = self.get_rect()?;

        Ok(window::Size {
            width: rect.right - rect.left,
            height: rect.bottom - rect.top,
        })
    }

    /// Retrieves the position of the window.
    pub fn get_position(&self) -> Result<window::Position, OSError> {
        let rect = self.get_rect()?;

        Ok(window::Position {
            x: rect.left,
            y: rect.top,
        })
    }

    /// Checks if the window is currently active.
    pub fn is_active(&self) -> Result<bool, OSError> {
        Ok(self.0 == unsafe { WindowsAndMessaging::GetForegroundWindow() })
    }
}
