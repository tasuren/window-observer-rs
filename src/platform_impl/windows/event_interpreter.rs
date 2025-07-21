use window_getter::{platform_impl::PlatformWindow, WindowId};
use wineventhook::{
    MaybeKnown, ObjectWindowEvent, SystemWindowEvent, WindowEvent, WindowEventType,
};

use crate::{
    platform_impl::PlatformError,
    window::{Position, Size},
    Event, EventFilter, EventTx, MaybeWindowAvailable, Window,
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

    fn dispatch(&self, window: Option<PlatformWindow>, event: Event) {
        if self.event_filter.should_dispatch(&event) {
            let payload = if let Some(window) = window {
                MaybeWindowAvailable::Available {
                    window: Window::new(window),
                    event,
                }
            } else {
                MaybeWindowAvailable::NotAvailable { event }
            };

            let _ = self.event_tx.send(Ok(payload));
        }
    }

    fn on_system_foreground_event(&mut self, window: PlatformWindow) -> Result<(), PlatformError> {
        let before_foreground = self.state.foreground.replace(window);

        if let Some(before_foreground) = before_foreground {
            if before_foreground.hwnd() != window.hwnd()
                && before_foreground.owner_pid()? == self.pid
            {
                self.dispatch(Some(before_foreground), Event::Backgrounded);
                self.dispatch(Some(before_foreground), Event::Unfocused);
            }
        }

        Ok(())
    }

    fn on_system_event(
        &mut self,
        window: PlatformWindow,
        event: SystemWindowEvent,
    ) -> Result<(), PlatformError> {
        match event {
            SystemWindowEvent::Foreground => {
                self.dispatch(Some(window), Event::Foregrounded);
                self.dispatch(Some(window), Event::Focused);
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
        match event {
            ObjectWindowEvent::LocationChange => {
                let Ok(visible_bounds) = window.visible_bounds() else {
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
                    self.dispatch(Some(window), Event::Moved);
                }

                // Check if the size has changed.
                let current_size: Size = visible_bounds.into();
                let previous_size = self.state.previous_size.replace(current_size.clone());

                if previous_size.is_none()
                    || previous_size.is_some_and(|previous_size| previous_size != current_size)
                {
                    self.dispatch(Some(window), Event::Resized);
                }
            }
            ObjectWindowEvent::Create => self.dispatch(Some(window), Event::Created),
            ObjectWindowEvent::Hide => self.dispatch(Some(window), Event::Hidden),
            ObjectWindowEvent::Show => self.dispatch(Some(window), Event::Showed),
            ObjectWindowEvent::Destroy => self.dispatch(
                None,
                Event::Closed {
                    window_id: WindowId::new(window.hwnd()),
                },
            ),
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
