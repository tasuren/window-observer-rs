use wineventhook::WindowEventHook;

use crate::{Error, EventTx};

use super::task::make_wineventhook_task;

pub struct WindowsWindowObserver {
    hook: WindowEventHook,
}

impl WindowsWindowObserver {
    pub async fn start(
        pid: i32,
        event_tx: EventTx,
        event_filter: crate::EventFilter,
    ) -> Result<Self, Error> {
        let hook = make_wineventhook_task(pid, event_tx, event_filter).await?;

        Ok(Self { hook })
    }

    pub async fn stop(self) -> Result<(), Error> {
        self.hook.unhook().await.map_err(super::OSError::from)?;

        Ok(())
    }
}
