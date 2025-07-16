use accessibility::{AXAttribute, AXUIElement};
use accessibility_sys::pid_t;

use super::{
    ax_function::ax_is_process_trusted,
    ax_observer::AXObserver,
    event_loop::{event_loop, get_event_loop, ObserverSource},
};
use crate::{
    platform_impl::macos::event::{
        dispatch_event_with_application_activated_notification,
        dispatch_event_with_window_related_notification, for_each_notification_event, FocusState,
    },
    Error, Event, EventFilter, EventTx,
};

/// Observes macOS window events and provides an interface to manage them.
/// This is wrapper of [`AXObserver`].
pub struct PlatformWindowObserver {
    source: ObserverSource,
    stopped: bool,
}

impl PlatformWindowObserver {
    /// Creates a new [`PlatformWindowObserver`] for a given process ID and event channel.
    pub async fn start(
        pid: i32,
        event_tx: EventTx,
        event_filter: EventFilter,
    ) -> Result<Self, Error> {
        if !ax_is_process_trusted() {
            return Err(Error::PermissionDenied);
        };

        // Instantiate `AXObserver`.
        let mut callback_state = ObserverCallbackState::default();
        let callback = move |ax_ui_element: AXUIElement, notification: String| {
            observer_callback(
                pid,
                ax_ui_element,
                event_tx.clone(),
                event_filter,
                notification,
                &mut callback_state,
            );
        };

        let observer =
            AXObserver::create(pid, Box::new(callback)).map_err(accessibility::Error::Ax)?;

        // Add the event filter to the observer.
        let app_element = AXUIElement::application(pid);

        for_each_notification_event(event_filter, |notification| {
            if let Err(ax_error) = observer.add_notification(&app_element, notification) {
                return Err::<_, Error>(match ax_error {
                    accessibility_sys::kAXErrorCannotComplete => Error::InvalidProcessId(pid as _),
                    accessibility_sys::kAXErrorNotificationUnsupported => Error::NotSupported,
                    accessibility_sys::kAXErrorNotificationAlreadyRegistered => return Ok(()),
                    ax_error => Error::PlatformSpecificError(accessibility::Error::Ax(ax_error)),
                });
            };

            Ok(())
        })?;

        // Wrap the observer in struct for preventing it from being dropped.
        let source = ObserverSource::new(observer);

        // Register the observer to the event loop. It will start receiving events.
        event_loop().await.register(source.get());

        Ok(Self {
            source,
            stopped: false,
        })
    }

    /// Stops the observer.
    pub async fn stop(mut self) {
        event_loop().await.unregister(self.source.get());
        self.stopped = true;
    }
}

impl Drop for PlatformWindowObserver {
    fn drop(&mut self) {
        if !self.stopped {
            // Unregister the observer in case the `stop` method was not called.
            get_event_loop()
                .expect("The event loop is not started.")
                .unregister(self.source.get());
            self.stopped = true;
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ObserverCallbackState {
    #[cfg(feature = "macos-private-api")]
    windows: std::collections::HashSet<u32>,
    focus_state: FocusState,
}

fn observer_callback(
    pid: pid_t,
    ax_ui_element: AXUIElement,
    event_tx: EventTx,
    event_filter: EventFilter,
    notification: String,
    state: &mut ObserverCallbackState,
) {
    let app_element = AXUIElement::application(pid);

    // Cache the current window IDs to dispatch closed events.
    #[cfg(feature = "macos-private-api")]
    let previous_window_ids = {
        use accessibility::AXUIElementAttributes;

        if let Ok(windows) = app_element.windows() {
            if notification == accessibility_sys::kAXWindowCreatedNotification
                || notification == accessibility_sys::kAXUIElementDestroyedNotification
            {
                let previous = std::mem::take(&mut state.windows);
                state.windows = windows
                    .into_iter()
                    .filter_map(|window| {
                        super::ax_function::ax_ui_element_get_window_id(&window).ok()
                    })
                    .collect::<std::collections::HashSet<_>>();

                Some(previous)
            } else {
                None
            }
        } else {
            return;
        }
    };

    // Extract the window element and send the event.
    let send_event = |event: Event| {
        if event_filter.should_dispatch(&event) {
            let _ = event_tx.send(event);
        }
    };

    match ax_ui_element.attribute(&AXAttribute::role()) {
        Ok(role) if role == accessibility_sys::kAXWindowRole => {
            dispatch_event_with_window_related_notification(
                ax_ui_element,
                send_event,
                &notification,
                &mut state.focus_state,
            );
        }
        Ok(_) => {
            // When application is activated or deactivated, we need to get the focused window
            // to dispatch the event `Event::Activated` or `Event::Deactivated`.
            // This is because `ax_ui_element` might not be a window element but an application element.

            let is_deactivated =
                notification == accessibility_sys::kAXApplicationDeactivatedNotification;

            if notification == accessibility_sys::kAXApplicationActivatedNotification
                || is_deactivated
            {
                dispatch_event_with_application_activated_notification(
                    app_element,
                    send_event,
                    is_deactivated,
                    &mut state.focus_state,
                );
            }
        }
        #[cfg(feature = "macos-private-api")]
        Err(accessibility::Error::Ax(accessibility_sys::kAXErrorInvalidUIElement)) => {
            // If the element is not a valid UI element, some window might be closed.
            use super::event::dispatch_event_with_ui_element_destroyed_notification;

            dispatch_event_with_ui_element_destroyed_notification(
                &previous_window_ids.unwrap(),
                &state.windows,
                send_event,
            );
        }
        _ => {}
    };
}
