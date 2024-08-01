use windows::Win32::{Foundation, UI::WindowsAndMessaging};

use super::{helper, OSError};
use crate::{window, Error};

impl Into<Error> for OSError {
    fn into(self) -> Error {
        Error::PlatformSpecificError(self)
    }
}

pub struct Window(pub Foundation::HWND);

impl Window {
    pub fn get_size(&self) -> Result<window::Size, Error> {
        let rect = self.get_rect()?;
        Ok(window::Size {
            width: rect.right - rect.left,
            height: rect.bottom - rect.top,
        })
    }

    pub fn get_position(&self) -> Result<window::Position, Error> {
        let rect = self.get_rect()?;
        Ok(window::Position {
            x: rect.left,
            y: rect.top,
        })
    }

    pub fn is_active(&self) -> Result<bool, Error> {
        Ok(self.0 == unsafe { WindowsAndMessaging::GetForegroundWindow() })
    }
}

pub trait WindowExtWindows {
    fn get_rect(&self) -> Result<Foundation::RECT, Error>;
}

impl WindowExtWindows for Window {
    fn get_rect(&self) -> Result<Foundation::RECT, Error> {
        helper::get_window_rect(self.0).map_err(Into::into)
    }
}
