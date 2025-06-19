use accessibility::{AXAttribute, AXUIElement};

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
        let callback = move |notification: String| {
            let ax_ui_element = AXUIElement::application(pid);
            observer_callback(ax_ui_element, event_tx.clone(), notification);
        };

        let observer = AXObserver::create(pid, Box::new(callback));

        // Add the event filter to the observer.
        for event in event_filter {
            observer
                .add_notification(&AXUIElement::application(pid), event.ax_notification())
                .expect("Failed to add notification to `AXObserver`");
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

fn observer_callback(ax_ui_element: AXUIElement, event_tx: EventTx, notification: String) {
    let Some(event) = Event::from_ax_notification(&notification) else {
        return;
    };

    let window_element = ax_ui_element
        .attribute(&AXAttribute::focused_window())
        .unwrap();
    let window = PlatformWindow::new(window_element);

    event_tx.send((crate::Window(window), event)).unwrap();
}
