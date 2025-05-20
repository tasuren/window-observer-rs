use super::{
    ax_function::ax_is_process_trusted,
    thread::{make_observe_thread, EventLoopController},
};
use crate::{EventFilter, EventTx};

/// Observes macOS window events and provides an interface to manage them.
/// This is wrapper of `AXObserver`.
pub struct MacOSWindowObserver {
    thread: std::thread::JoinHandle<()>,
    controller: EventLoopController,
}

impl MacOSWindowObserver {
    /// Creates a new `MacOSWindowObserver` for a given process ID and event channel.
    pub async fn start(
        pid: i32,
        event_tx: EventTx,
        event_filter: EventFilter,
    ) -> Result<Self, crate::Error> {
        if !ax_is_process_trusted() {
            return Err(crate::Error::PermissinoDenied);
        };

        let (controller_tx, controller_rx) = tokio::sync::oneshot::channel();
        let thread = make_observe_thread(pid, event_tx, event_filter, controller_tx);
        let controller = controller_rx.await.unwrap();

        Ok(Self { thread, controller })
    }

    /// Stops the observer.
    pub async fn stop(self) {
        self.controller.stop();

        tokio::task::spawn_blocking(|| self.thread.join().unwrap())
            .await
            .unwrap();
    }
}
