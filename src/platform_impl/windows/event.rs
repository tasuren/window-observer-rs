use wineventhook::{
    MaybeKnown, ObjectWindowEvent, SystemWindowEvent, WindowEvent, WindowEventType,
};

use crate::Event;

/// Converts a [`WindowEvent`] into a library-specific [`Event`].
pub fn make_event(event: WindowEvent) -> Option<Event> {
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
