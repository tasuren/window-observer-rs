use windows::Win32::{Foundation, UI::WindowsAndMessaging};

use super::{helper, OSError};
use crate::window;

pub struct WindowsWindow(Foundation::HWND);
unsafe impl Send for WindowsWindow {}
unsafe impl Sync for WindowsWindow {}

impl WindowsWindow {
    pub fn new(hwnd: Foundation::HWND) -> Self {
        Self(hwnd)
    }

    pub fn hwnd(&self) -> Foundation::HWND {
        self.0
    }

    pub fn get_rect(&self) -> Result<Foundation::RECT, OSError> {
        Ok(helper::get_window_rect(self.0)?)
    }

    pub fn get_size(&self) -> Result<window::Size, OSError> {
        let rect = self.get_rect()?;

        Ok(window::Size {
            width: rect.right - rect.left,
            height: rect.bottom - rect.top,
        })
    }

    pub fn get_position(&self) -> Result<window::Position, OSError> {
        let rect = self.get_rect()?;

        Ok(window::Position {
            x: rect.left,
            y: rect.top,
        })
    }

    pub fn is_active(&self) -> Result<bool, OSError> {
        Ok(self.0 == unsafe { WindowsAndMessaging::GetForegroundWindow() })
    }
}
