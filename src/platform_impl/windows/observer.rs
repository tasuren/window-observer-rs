use tokio::sync::oneshot;
use wineventhook::WindowEventHook;

use crate::{Error, EventTx};

use super::hook_task::make_wineventhook_task;

/// Observes window events on the Windows platform by using [wineventhook].
pub struct WindowsWindowObserver {
    hook: WindowEventHook,
    done_rx: oneshot::Receiver<()>,
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

        let (hook, done_rx) = make_wineventhook_task(pid, event_tx, event_filter).await?;

        Ok(Self { hook, done_rx })
    }

    /// Stops observing window events.
    pub async fn stop(self) -> Result<(), Error> {
        self.hook
            .unhook()
            .await
            .map_err(super::error::WindowsError::from)?;

        self.done_rx
            .await
            .map_err(super::error::WindowsError::from)?;

        Ok(())
    }

    /// Retrieves the underlying [`WindowEventHook`].
    pub fn hook(&self) -> &WindowEventHook {
        &self.hook
    }
}
