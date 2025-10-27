use wineventhook::WindowEventHook;

use crate::{Error, EventTx};

use super::hook_task::make_wineventhook_task;

/// Observes window events on the Windows platform by using [wineventhook].
pub struct WindowsWindowObserver {
    hook: WindowEventHook,
}

impl WindowsWindowObserver {
    /// Starts observing window events for a specific process ID.
    pub async fn start(
        pid: u32,
        event_tx: EventTx,
        event_filter: crate::EventFilter,
    ) -> Result<Self, Error> {
        if pid == 0 {
            return Err(Error::InvalidProcessId(pid));
        }

        let hook = make_wineventhook_task(pid, event_tx, event_filter).await?;

        Ok(Self { hook })
    }

    /// Stops observing window events.
    pub async fn stop(self) -> Result<(), Error> {
        self.hook
            .unhook()
            .await
            .map_err(super::error::WindowsError::from)?;

        Ok(())
    }

    /// Retrieves the underlying [`WindowEventHook`].
    pub fn hook(&self) -> &WindowEventHook {
        &self.hook
    }
}
