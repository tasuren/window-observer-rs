use crate::{
    platform_impl::PlatformWindow, Event, EventFilter, EventTx, MaybeWindowAvailable, Window,
};
use accessibility::{AXUIElement, AXUIElementAttributes};

#[inline]
fn create_window_unchecked(element: AXUIElement) -> Window {
    Window::new(PlatformWindow::new(element))
}

#[derive(Default, Clone, Debug)]
struct EventInterpreterState {
    #[cfg(feature = "macos-private-api")]
    current_window_ids: std::collections::HashSet<u32>,
    #[cfg(feature = "macos-private-api")]
    previous_window_ids: std::collections::HashSet<u32>,
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

    fn dispatch(&self, window: Option<Window>, event: Event) {
        if self.event_filter.should_dispatch(&event) {
            let payload = if let Some(window) = window {
                MaybeWindowAvailable::Available { window, event }
            } else {
                MaybeWindowAvailable::NotAvailable { event }
            };

            let _ = self.event_tx.send(Ok(payload));
        }
    }

    fn on_application_activated_or_deactivated(
        &mut self,
        is_deactivated: bool,
    ) -> Result<(), accessibility::Error> {
        for element in self.app_element.windows()?.iter() {
            let window = create_window_unchecked(element.clone());

            if is_deactivated {
                self.dispatch(Some(window), Event::Backgrounded);
            } else {
                self.dispatch(Some(window), Event::Foregrounded);
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
            .filter_map(|window| {
                super::binding_ax_function::ax_ui_element_get_window_id(&window).ok()
            })
            .collect::<std::collections::HashSet<_>>();
        self.state.previous_window_ids =
            std::mem::replace(&mut self.state.current_window_ids, current);

        Ok(())
    }

    pub fn on_window_created(&mut self, element: AXUIElement) -> Result<(), accessibility::Error> {
        let window = create_window_unchecked(element);
        self.dispatch(Some(window), Event::Created);

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
            self.dispatch(None, event);
        }

        Ok(())
    }

    pub fn on_window_moved(&self, element: AXUIElement) {
        let window = create_window_unchecked(element);
        self.dispatch(Some(window), Event::Moved);
    }

    pub fn on_window_resized(&self, element: AXUIElement) {
        let window = create_window_unchecked(element);
        self.dispatch(Some(window), Event::Resized);
    }

    pub fn on_focused_window_changed(
        &mut self,
        element: AXUIElement,
    ) -> Result<(), accessibility::Error> {
        for maybe_backgrounded in self.app_element.windows()?.iter() {
            if *maybe_backgrounded != element {
                let window = create_window_unchecked(maybe_backgrounded.clone());
                self.dispatch(Some(window), Event::Backgrounded);
            }
        }

        let window = create_window_unchecked(element.clone());
        self.dispatch(Some(window), Event::Foregrounded);

        // If focused window is changed, we should also dispatch the unfocused event.
        if let Some(previous_window_element) = self
            .state
            .previous_focused_window
            .replace(element.clone())
            .and_then(|e| (e != element).then_some(e))
        {
            let window = create_window_unchecked(previous_window_element);
            self.dispatch(Some(window), Event::Unfocused);
        }

        let window = create_window_unchecked(element);
        self.dispatch(Some(window), Event::Focused);

        Ok(())
    }

    pub fn on_window_miniaturized(&mut self, element: AXUIElement) {
        let window = create_window_unchecked(element.clone());
        self.dispatch(Some(window), Event::Hidden);
    }

    pub fn on_window_deminimized(&mut self, element: AXUIElement) {
        let window = create_window_unchecked(element);
        self.dispatch(Some(window.clone()), Event::Showed);
        self.dispatch(Some(window), Event::Foregrounded);
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
            #[cfg(feature = "macos-private-api")]
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
                self.on_focused_window_changed(element)?;
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
    if event_filter.focused {
        f(accessibility_sys::kAXFocusedWindowChangedNotification)?;
    }

    if event_filter.unfocused {
        f(accessibility_sys::kAXFocusedWindowChangedNotification)?;
    }

    if event_filter.foregrounded {
        f(accessibility_sys::kAXApplicationActivatedNotification)?;
        f(accessibility_sys::kAXFocusedWindowChangedNotification)?;
        f(accessibility_sys::kAXWindowDeminiaturizedNotification)?;
    }

    if event_filter.backgrounded {
        f(accessibility_sys::kAXApplicationDeactivatedNotification)?;
        f(accessibility_sys::kAXFocusedWindowChangedNotification)?;
    }

    if event_filter.hide {
        f(accessibility_sys::kAXWindowMiniaturizedNotification)?;
    }

    if event_filter.showed {
        f(accessibility_sys::kAXWindowDeminiaturizedNotification)?;
    }

    if event_filter.moved {
        f(accessibility_sys::kAXWindowMovedNotification)?;
    }

    if event_filter.resized {
        f(accessibility_sys::kAXWindowResizedNotification)?;
    }

    if event_filter.created {
        f(accessibility_sys::kAXWindowCreatedNotification)?;
    }

    if event_filter.closed {
        f(accessibility_sys::kAXUIElementDestroyedNotification)?;
    }

    Ok(())
}
