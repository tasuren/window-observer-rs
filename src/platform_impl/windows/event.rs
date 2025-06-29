use window_getter::platform_impl::PlatformWindow;
use wineventhook::{
    MaybeKnown, ObjectWindowEvent, SystemWindowEvent, WindowEvent, WindowEventType,
};

use crate::Event;

/// Structs for conversion between a [`WindowEvent`] and a library-specific [`Event`].
pub struct EventManager {
    pid: u32,
    foreground: Option<PlatformWindow>,
}

impl EventManager {
    pub fn new(pid: u32) -> Self {
        Self {
            pid,
            foreground: None,
        }
    }

    pub fn convert_event(
        &mut self,
        window: PlatformWindow,
        event: WindowEvent,
    ) -> Option<(Event, PlatformWindow)> {
        if matches!(
            event.event_type(),
            WindowEventType::System(MaybeKnown::Known(SystemWindowEvent::Foreground))
        ) {
            let before_foreground = self.foreground.take();
            self.foreground = Some(window);

            if let Some(before_foreground) = before_foreground {
                if before_foreground.hwnd() != window.hwnd()
                    && before_foreground.owner_pid().ok()? == self.pid
                {
                    return Some((Event::Deactivated, before_foreground));
                }
            }
        }

        if window.owner_pid().ok()? != self.pid {
            return None;
        }

        if let WindowEventType::System(MaybeKnown::Known(event)) = event.event_type() {
            return Some((
                match event {
                    SystemWindowEvent::MoveSizeEnd => Event::Resized,
                    SystemWindowEvent::Foreground => Event::Activated,
                    _ => return None,
                },
                window,
            ));
        };

        if let WindowEventType::Object(MaybeKnown::Known(event)) = event.event_type() {
            return Some((
                match event {
                    ObjectWindowEvent::LocationChange => Event::Moved,
                    _ => return None,
                },
                window,
            ));
        };

        None
    }
}
