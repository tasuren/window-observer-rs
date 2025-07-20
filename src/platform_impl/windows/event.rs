use window_getter::{platform_impl::PlatformWindow, WindowId};
use wineventhook::{
    MaybeKnown, ObjectWindowEvent, SystemWindowEvent, WindowEvent, WindowEventType,
};

use crate::{
    platform_impl::PlatformError,
    window::{Position, Size},
    Event, EventFilter, EventTx, Window,
};

#[derive(Debug, Default, Clone)]
struct EventInterpreterState {
    foreground: Option<PlatformWindow>,
    previous_pos: Option<Position>,
    previous_size: Option<Size>,
}

/// Structs for conversion between a [`WindowEvent`] and a library-specific [`Event`].
pub struct EventInterpreter {
    pid: u32,
    event_tx: EventTx,
    event_filter: EventFilter,
    state: EventInterpreterState,
}

impl EventInterpreter {
    pub fn new(pid: u32, event_tx: EventTx, event_filter: EventFilter) -> Self {
        Self {
            pid,
            event_tx,
            event_filter,
            state: Default::default(),
        }
    }

    fn dispatch(&self, event: Event) {
        if self.event_filter.should_dispatch(&event) {
            let _ = self.event_tx.send(Ok(event));
        }
    }

    fn on_system_foreground_event(&mut self, window: PlatformWindow) -> Result<(), PlatformError> {
        let before_foreground = self.state.foreground.replace(window);

        if let Some(before_foreground) = before_foreground {
            if before_foreground.hwnd() != window.hwnd()
                && before_foreground.owner_pid()? == self.pid
            {
                let event = Event::Backgrounded {
                    window: Window::new(before_foreground),
                };
                self.dispatch(event);

                let event = Event::Unfocused {
                    window: Window::new(before_foreground),
                };
                self.dispatch(event);
            }
        }

        Ok(())
    }

    fn on_system_event(
        &mut self,
        window: PlatformWindow,
        event: SystemWindowEvent,
    ) -> Result<(), PlatformError> {
        let window = Window::new(window);
        match event {
            SystemWindowEvent::Foreground => {
                self.dispatch(Event::Foregrounded {
                    window: window.clone(),
                });
                self.dispatch(Event::Focused { window });
            }
            _ => return Ok(()),
        };

        Ok(())
    }

    fn on_object_event(
        &mut self,
        window: PlatformWindow,
        event: ObjectWindowEvent,
    ) -> Result<(), PlatformError> {
        let window = Window::new(window);

        match event {
            ObjectWindowEvent::LocationChange => {
                let Ok(visible_bounds) = window.inner().visible_bounds() else {
                    return Ok(());
                };

                // Check if the position has changed.
                // `LocationChange` can be triggered by both position and size changes.
                let current_pos: Position = visible_bounds.clone().into();
                let previous_pos = self.state.previous_pos.replace(current_pos.clone());

                if previous_pos.is_none()
                    || previous_pos
                        .as_ref()
                        .is_some_and(|previous_pos| *previous_pos != current_pos)
                {
                    self.dispatch(Event::Moved {
                        window: window.clone(),
                    });
                }

                // Check if the size has changed.
                let current_size: Size = visible_bounds.into();
                let previous_size = self.state.previous_size.replace(current_size.clone());

                if previous_size.is_none()
                    || previous_size.is_some_and(|previous_size| previous_size != current_size)
                {
                    self.dispatch(Event::Resized { window });
                }
            }
            ObjectWindowEvent::Create => self.dispatch(Event::Created { window }),
            ObjectWindowEvent::Destroy => self.dispatch(Event::Closed {
                window_id: WindowId::new(window.inner().hwnd()),
            }),
            _ => return Ok(()),
        };

        Ok(())
    }

    fn dispatch_wineventhook_event(
        &mut self,
        window: PlatformWindow,
        event: WindowEvent,
    ) -> Result<(), PlatformError> {
        if matches!(
            event.event_type(),
            WindowEventType::System(MaybeKnown::Known(SystemWindowEvent::Foreground))
        ) {
            self.on_system_foreground_event(window)?;
        }

        if window.owner_pid()? != self.pid {
            return Ok(());
        }

        match event.event_type() {
            WindowEventType::System(MaybeKnown::Known(event)) => {
                self.on_system_event(window, event)?;
            }
            WindowEventType::Object(MaybeKnown::Known(event)) => {
                self.on_object_event(window, event)?;
            }
            _ => {}
        }

        Ok(())
    }

    pub fn interpret_wineventhook_event(&mut self, window: PlatformWindow, event: WindowEvent) {
        if let Err(e) = self.dispatch_wineventhook_event(window, event) {
            let _ = self.event_tx.send(Err(e));
        };
    }
}
