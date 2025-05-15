use accessibility_sys::{
    kAXApplicationActivatedNotification, kAXMovedNotification, kAXResizedNotification,
};

use crate::Event;

pub trait EventMacOSExt {
    fn from_ax_notification(notification: &str) -> Option<Event>;
    fn ax_notification(&self) -> &'static str;
}

impl EventMacOSExt for Event {
    fn from_ax_notification(notification: &str) -> Option<Self> {
        Some(match notification {
            accessibility_sys::kAXApplicationActivatedNotification => Event::Activated,
            accessibility_sys::kAXMovedNotification => Event::Moved,
            accessibility_sys::kAXResizedNotification => Event::Resized,
            _ => return None,
        })
    }

    fn ax_notification(&self) -> &'static str {
        match *self {
            Event::Activated => kAXApplicationActivatedNotification,
            Event::Moved => kAXMovedNotification,
            Event::Resized => kAXResizedNotification,
        }
    }
}
