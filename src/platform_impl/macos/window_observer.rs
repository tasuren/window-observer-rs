use accessibility::{AXAttribute, AXUIElement};
use accessibility_sys::pid_t;

use super::{
    ax_function::ax_is_process_trusted,
    ax_observer::AXObserver,
    event::EventMacOSExt,
    event_loop::{event_loop, get_event_loop, ObserverSource},
    window::PlatformWindow,
};
use crate::{Error, Event, EventFilter, EventTx};

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
        let callback = move |ax_ui_element: AXUIElement, notification: String| {
            observer_callback(pid, ax_ui_element, event_tx.clone(), notification);
        };

        let observer =
            AXObserver::create(pid, Box::new(callback)).map_err(accessibility::Error::Ax)?;

        // Add the event filter to the observer.
        for event in event_filter {
            if let Err(ax_error) =
                observer.add_notification(&AXUIElement::application(pid), event.ax_notification())
            {
                return Err(match ax_error {
                    accessibility_sys::kAXErrorCannotComplete => Error::InvalidProcessId(pid as _),
                    accessibility_sys::kAXErrorNotificationUnsupported => Error::NotSupported,
                    ax_error => Error::PlatformSpecificError(accessibility::Error::Ax(ax_error)),
                });
            };
        }

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

fn observer_callback(
    pid: pid_t,
    ax_ui_element: AXUIElement,
    event_tx: EventTx,
    notification: String,
) {
    let Some(event) = Event::from_ax_notification(&notification) else {
        return;
    };

    let window_element = match ax_ui_element.attribute(&AXAttribute::role()) {
        Ok(role) if role == accessibility_sys::kAXWindowRole => ax_ui_element,

        // When application is activated or deactivated, we need to get the focused window
        // to dispatch the event `Event::Activated` or `Event::Deactivated`.
        Ok(role)
            if role == accessibility_sys::kAXApplicationActivatedNotification
                || role == accessibility_sys::kAXApplicationDeactivatedNotification =>
        {
            match ax_ui_element.attribute(&AXAttribute::focused_window()) {
                Ok(element) => element,
                Err(accessibility::Error::Ax(accessibility_sys::kAXErrorNoValue)) => {
                    // If the focused window is not available, the application might not have a window.
                    return;
                }
                Err(e) => panic!("Failed to get focused window: {e:?}"),
            }
        }

        Err(accessibility::Error::Ax(accessibility_sys::kAXErrorInvalidUIElement)) => {
            // If the element is not a valid UI element, some window might be closed.
            ax_ui_element
        }

        _ => return,
    };

    // SAFETY: The `window_element` is guaranteed to be a valid `AXUIElement` representing a window.
    let window = unsafe { PlatformWindow::new_unchecked(window_element) };
    let window = crate::Window(window);
    event_tx.send((window, event)).unwrap();
}
