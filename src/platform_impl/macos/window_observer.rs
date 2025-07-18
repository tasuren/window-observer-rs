use accessibility::AXUIElement;

use super::{
    binding_ax_function::ax_is_process_trusted,
    binding_ax_observer::AXObserver,
    event_loop::{event_loop, get_event_loop, ObserverSource},
};
use crate::{
    platform_impl::macos::event::{for_each_notification_event, EventInterpreter},
    Error, EventFilter, EventTx,
};

/// Observes macOS window events and provides an interface to manage them.
/// This is wrapper of [`AXObserver`].
pub struct MacOSWindowObserver {
    source: ObserverSource,
    stopped: bool,
}

impl MacOSWindowObserver {
    /// Creates a new `MacOSWindowObserver` for a given process ID and event channel.
    pub async fn start(
        pid: accessibility_sys::pid_t,
        event_tx: EventTx,
        event_filter: EventFilter,
    ) -> Result<Self, Error> {
        if !ax_is_process_trusted() {
            return Err(Error::PermissionDenied);
        };

        // Instantiate `AXObserver`.
        let mut event_interpreter =
            EventInterpreter::new(AXUIElement::application(pid), event_tx, event_filter);
        let callback = move |element: AXUIElement, notification: String| {
            event_interpreter.interpret_ax_notification(element, &notification);
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

impl Drop for MacOSWindowObserver {
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
