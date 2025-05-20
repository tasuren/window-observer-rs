use std::mem::MaybeUninit;

use windows::Win32::{Foundation, UI::WindowsAndMessaging};
use wineventhook::{MaybeKnown, ObjectWindowEvent, SystemWindowEvent, WindowEventType};

use crate::Event;

pub fn make_event(event: wineventhook::WindowEvent) -> Option<Event> {
    if let WindowEventType::System(MaybeKnown::Known(event)) = event.event_type() {
        return Some(match event {
            SystemWindowEvent::MoveSizeEnd => Event::Resized,
            SystemWindowEvent::Foreground => Event::Activated,
            _ => return None,
        });
    };

    if let WindowEventType::Object(MaybeKnown::Known(event)) = event.event_type() {
        return Some(match event {
            ObjectWindowEvent::LocationChange => Event::Moved,
            _ => return None,
        });
    };

    None
}

pub fn get_window_rect(hwnd: Foundation::HWND) -> windows::core::Result<Foundation::RECT> {
    let mut value = MaybeUninit::uninit();

    unsafe {
        WindowsAndMessaging::GetWindowRect(hwnd, value.as_mut_ptr())?;
        Ok(value.assume_init())
    }
}
