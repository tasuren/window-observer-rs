use accessibility_sys::{
    kAXApplicationActivatedNotification, kAXMovedNotification, kAXResizedNotification,
};

use crate::Event;

/// An extension trait for the `Event` enum to handle macOS-specific notifications.
pub trait EventMacOSExt {
    /// Converts a macOS accessibility notification string into an `Event`.
    ///
    /// # Parameters
    /// - `notification`: The macOS notification string.
    ///
    /// # Returns
    /// An `Option` containing the corresponding `Event`, or `None` if the notification is not recognized.
    fn from_ax_notification(notification: &str) -> Option<Event>;

    /// Converts an `Event` into its corresponding macOS accessibility notification string.
    ///
    /// # Returns
    /// A static string representing the macOS notification.
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
