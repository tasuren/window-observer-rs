use std::collections::HashSet;

use accessibility::{AXUIElement, AXUIElementAttributes};

use crate::{platform_impl::PlatformWindow, Event, EventFilter, Window};

pub(crate) fn dispatch_event_with_application_activated_notification(
    app_element: AXUIElement,
    dispatch: impl Fn(Event),
    is_deactivated: bool,
    state: &mut FocusState,
) {
    if let Ok(element) = app_element.focused_window() {
        let window = Window::new(PlatformWindow::new(element));

        if is_deactivated {
            dispatch(Event::Unfocused { window });
        } else {
            state.previous = Some(window.clone());
            dispatch(Event::Focused { window });
        };
    }

    if let Ok(elements) = app_element.windows() {
        for element in elements.iter() {
            let window = Window::new(PlatformWindow::new(element.clone()));
            if is_deactivated {
                dispatch(Event::Backgrounded { window });
            } else {
                dispatch(Event::Foregrounded { window });
            }
        }
    }
}

pub(crate) fn dispatch_event_with_ui_element_destroyed_notification(
    previous_window_ids: &HashSet<u32>,
    current_window_ids: &HashSet<u32>,
    dispatch: impl Fn(Event),
) {
    let removed = previous_window_ids.difference(current_window_ids);

    for window_id in removed.cloned() {
        let event = Event::Closed {
            window_id: window_id.into(),
        };
        dispatch(event);
    }
}

#[derive(Default, Debug, Clone)]
pub(crate) struct FocusState {
    previous: Option<Window>,
}

pub(crate) fn dispatch_event_with_window_related_notification(
    window_element: AXUIElement,
    dispatch: impl Fn(Event),
    notification: &str,
    state: &mut FocusState,
) {
    let window = Window::new(PlatformWindow::new(window_element));

    match notification {
        accessibility_sys::kAXWindowCreatedNotification => {
            if window.is_focused().is_ok_and(|focused| focused) {
                dispatch(Event::Focused {
                    window: window.clone(),
                });
                state.previous = Some(window.clone());
            }

            dispatch(Event::Created { window });
        }
        accessibility_sys::kAXMovedNotification => dispatch(Event::Moved { window }),
        accessibility_sys::kAXResizedNotification => dispatch(Event::Resized { window }),
        accessibility_sys::kAXFocusedWindowChangedNotification => {
            dispatch(Event::Focused {
                window: window.clone(),
            });
            dispatch(Event::Foregrounded {
                window: window.clone(),
            });

            let previous_focused_window = state.previous.replace(window);

            if let Some(window) = previous_focused_window {
                dispatch(Event::Unfocused {
                    window: window.clone(),
                });
                dispatch(Event::Backgrounded { window });
            }
        }
        accessibility_sys::kAXWindowMiniaturizedNotification => {
            dispatch(Event::Unfocused {
                window: window.clone(),
            });
            dispatch(Event::Backgrounded { window });
        }
        accessibility_sys::kAXWindowDeminiaturizedNotification => {
            state.previous = Some(window.clone());

            dispatch(Event::Focused {
                window: window.clone(),
            });
            dispatch(Event::Foregrounded { window });
        }
        _ => {}
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
        f(accessibility_sys::kAXWindowCreatedNotification)?;
    }

    if event_filter.backgrounded || event_filter.unfocused {
        f(accessibility_sys::kAXApplicationDeactivatedNotification)?;
        f(accessibility_sys::kAXFocusedWindowChangedNotification)?;
        f(accessibility_sys::kAXWindowDeminiaturizedNotification)?;
        f(accessibility_sys::kAXWindowCreatedNotification)?;
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
