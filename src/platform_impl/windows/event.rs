use wineventhook::{MaybeKnown, ObjectWindowEvent, SystemWindowEvent, WindowEventType};

use crate::Event;

/// Converts a `wineventhook::WindowEvent` into a library-specific `Event`.
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
