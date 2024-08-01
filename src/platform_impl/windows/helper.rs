use std::mem::MaybeUninit;

use smallvec::{smallvec, SmallVec};
use windows::Win32::{Foundation, UI::WindowsAndMessaging};
use wineventhook::SystemWindowEvent;

use crate::Event;

pub fn make_event(event: wineventhook::WindowEvent) -> SmallVec<[Event; 2]> {
    if let wineventhook::WindowEventType::System(event) = event.event_type() {
        if let wineventhook::MaybeKnown::Known(event) = event {
            return match event {
                SystemWindowEvent::MoveSizeEnd => smallvec![Event::Moved, Event::Resized],
                SystemWindowEvent::Foreground => smallvec![Event::Activated],
                _ => SmallVec::default(),
            };
        };
    };

    SmallVec::default()
}

pub fn get_window_rect(hwnd: Foundation::HWND) -> Result<Foundation::RECT, windows::core::Error> {
    let mut value = MaybeUninit::uninit();
    unsafe {
        WindowsAndMessaging::GetWindowRect(hwnd, value.as_mut_ptr())?;
        Ok(value.assume_init())
    }
}
