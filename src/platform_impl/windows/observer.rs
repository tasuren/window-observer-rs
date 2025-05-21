use wineventhook::WindowEventHook;

use crate::{Error, EventTx};

use super::task::make_wineventhook_task;

/// Observes window events on the Windows platform.
pub struct WindowsWindowObserver {
    hook: WindowEventHook,
}

impl WindowsWindowObserver {
    /// Starts observing window events for a specific process ID.
    ///
    /// # Parameters
    /// - `pid`: The process ID to observe.
    /// - `event_tx`: The channel to send observed events.
    /// - `event_filter`: The filter to apply to observed events.
    pub async fn start(
        pid: i32,
        event_tx: EventTx,
        event_filter: crate::EventFilter,
    ) -> Result<Self, Error> {
        let hook = make_wineventhook_task(pid, event_tx, event_filter).await?;

        Ok(Self { hook })
    }

    /// Stops observing window events.
    pub async fn stop(self) -> Result<(), Error> {
        self.hook.unhook().await.map_err(super::OSError::from)?;

        Ok(())
    }
}
