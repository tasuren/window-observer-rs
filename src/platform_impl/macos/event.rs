use std::collections::HashSet;

use accessibility::{AXUIElement, AXUIElementAttributes};

use crate::{platform_impl::PlatformWindow, Event, EventFilter, EventTx, Window};

#[inline]
fn create_window_unchecked(element: AXUIElement) -> Window {
    Window::new(unsafe { PlatformWindow::new_unchecked(element) })
}

#[derive(Default, Clone, Debug)]
struct EventInterpreterState {
    #[cfg(feature = "macos-private-api")]
    current_window_ids: HashSet<u32>,
    #[cfg(feature = "macos-private-api")]
    previous_window_ids: HashSet<u32>,
    previous_focused_window: Option<AXUIElement>,
}

pub(crate) struct EventInterpreter {
    app_element: AXUIElement,
    event_tx: EventTx,
    event_filter: EventFilter,
    state: EventInterpreterState,
}

impl EventInterpreter {
    pub fn new(app_element: AXUIElement, event_tx: EventTx, event_filter: EventFilter) -> Self {
        Self {
            app_element,
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

    fn dispatch_focused(&mut self, window_element: AXUIElement) {
        // If focused window is changed, we should also dispatch the unfocused event.
        if let Some(element) = self
            .state
            .previous_focused_window
            .replace(window_element.clone())
            .and_then(|e| (e != window_element).then_some(e))
        {
            let window = create_window_unchecked(element);
            self.dispatch(Event::Unfocused { window });
        }

        let window = create_window_unchecked(window_element);
        self.dispatch(Event::Focused { window });
    }

    fn dispatch_unfocused(&mut self, window_element: AXUIElement) {
        if self
            .state
            .previous_focused_window
            .as_ref()
            .is_some_and(|e| *e == window_element)
        {
            self.state.previous_focused_window = None;
        }

        let window = create_window_unchecked(window_element);
        self.dispatch(Event::Unfocused { window });
    }

    fn on_application_activated_or_deactivated(
        &mut self,
        is_deactivated: bool,
    ) -> Result<(), accessibility::Error> {
        if !is_deactivated {
            self.dispatch_focused(self.app_element.focused_window()?);
        };

        for element in self.app_element.windows()?.iter() {
            let window = create_window_unchecked(element.clone());

            if is_deactivated {
                self.dispatch(Event::Backgrounded { window });

                // If previously focused window is the same as the current element,
                // it means that the window is being unfocused because
                // the application is deactivated.
                if self
                    .state
                    .previous_focused_window
                    .as_ref()
                    .is_some_and(|e| *e == *element)
                {
                    self.dispatch_unfocused(element.clone());
                }
            } else {
                self.dispatch(Event::Foregrounded { window });
            }
        }

        Ok(())
    }

    fn on_application_activated(&mut self) -> Result<(), accessibility::Error> {
        self.on_application_activated_or_deactivated(false)
    }

    fn on_application_deactivated(&mut self) -> Result<(), accessibility::Error> {
        self.on_application_activated_or_deactivated(true)
    }

    #[cfg(feature = "macos-private-api")]
    fn refresh_window_ids_state(&mut self) -> Result<(), accessibility::Error> {
        let current = self
            .app_element
            .windows()?
            .into_iter()
            .filter_map(|window| super::ax_function::ax_ui_element_get_window_id(&window).ok())
            .collect::<std::collections::HashSet<_>>();
        self.state.previous_window_ids =
            std::mem::replace(&mut self.state.current_window_ids, current);

        Ok(())
    }

    pub fn on_window_created(&mut self, element: AXUIElement) -> Result<(), accessibility::Error> {
        let window = create_window_unchecked(element);
        self.dispatch(Event::Created { window });

        // Track the windows currently known to the application.
        #[cfg(feature = "macos-private-api")]
        {
            self.refresh_window_ids_state()?;
        }

        Ok(())
    }

    #[cfg(feature = "macos-private-api")]
    pub fn on_ui_element_destroyed(&mut self) -> Result<(), accessibility::Error> {
        self.refresh_window_ids_state()?;

        let removed = self
            .state
            .previous_window_ids
            .difference(&self.state.current_window_ids);

        for window_id in removed.cloned() {
            let event = Event::Closed {
                window_id: window_id.into(),
            };
            self.dispatch(event);
        }

        Ok(())
    }

    pub fn on_window_moved(&self, element: AXUIElement) {
        let window = create_window_unchecked(element);
        self.dispatch(Event::Moved { window });
    }

    pub fn on_window_resized(&self, element: AXUIElement) {
        let window = create_window_unchecked(element);
        self.dispatch(Event::Resized { window });
    }

    pub fn on_focused_window_changed(&mut self, element: AXUIElement) {
        self.dispatch_focused(element.clone());
        let window = create_window_unchecked(element);
        self.dispatch(Event::Foregrounded { window });
    }

    pub fn on_window_miniaturized(&self, element: AXUIElement) {
        let window = create_window_unchecked(element);
        self.dispatch(Event::Backgrounded { window });
    }

    pub fn on_window_deminimized(&mut self, element: AXUIElement) {
        self.dispatch_focused(element.clone());
        let window = create_window_unchecked(element);
        self.dispatch(Event::Foregrounded { window });
    }

    fn dispatch_ax_notification(
        &mut self,
        element: AXUIElement,
        notification: &str,
    ) -> Result<bool, accessibility::Error> {
        match notification {
            accessibility_sys::kAXWindowCreatedNotification => {
                self.on_window_created(element)?;
            }
            accessibility_sys::kAXUIElementDestroyedNotification => {
                self.on_ui_element_destroyed()?;
            }
            accessibility_sys::kAXWindowResizedNotification => {
                self.on_window_resized(element);
            }
            accessibility_sys::kAXWindowMovedNotification => {
                self.on_window_moved(element);
            }
            accessibility_sys::kAXApplicationActivatedNotification => {
                self.on_application_activated()?;
            }
            accessibility_sys::kAXApplicationDeactivatedNotification => {
                self.on_application_deactivated()?;
            }
            accessibility_sys::kAXFocusedWindowChangedNotification => {
                self.on_focused_window_changed(element);
            }
            accessibility_sys::kAXWindowMiniaturizedNotification => {
                self.on_window_miniaturized(element);
            }
            accessibility_sys::kAXWindowDeminiaturizedNotification => {
                self.on_window_deminimized(element);
            }
            _ => return Ok(false),
        }

        Ok(true)
    }

    pub fn interpret_ax_notification(&mut self, element: AXUIElement, notification: &str) -> bool {
        match self.dispatch_ax_notification(element, notification) {
            Ok(dispatched) => dispatched,
            Err(e) => {
                let _ = self.event_tx.send(Err(e));
                false
            }
        }
    }
}

/// Iterates over the event filter and calls the provided function
/// for each notification name on Accessibility API.
pub(crate) fn for_each_notification_event<E>(
    event_filter: EventFilter,
    mut f: impl FnMut(&'static str) -> Result<(), E>,
) -> Result<(), E> {
    if event_filter.focused || event_filter.foregrounded {
        f(accessibility_sys::kAXApplicationActivatedNotification)?;
        f(accessibility_sys::kAXFocusedWindowChangedNotification)?;
        f(accessibility_sys::kAXWindowMiniaturizedNotification)?;
    }

    if event_filter.backgrounded {
        f(accessibility_sys::kAXApplicationDeactivatedNotification)?;
        f(accessibility_sys::kAXFocusedWindowChangedNotification)?;
        f(accessibility_sys::kAXWindowMiniaturizedNotification)?;
    }

    if event_filter.moved {
        f(accessibility_sys::kAXMovedNotification)?;
    }

    if event_filter.resized {
        f(accessibility_sys::kAXResizedNotification)?;
    }

    if event_filter.created {
        f(accessibility_sys::kAXWindowCreatedNotification)?;
    }

    if event_filter.closed {
        f(accessibility_sys::kAXUIElementDestroyedNotification)?;
    }

    Ok(())
}
