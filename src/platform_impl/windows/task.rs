use tokio::sync::mpsc::UnboundedReceiver;
use windows::Win32::Foundation;
use wineventhook::{raw_event, WindowEventHook};

use crate::{EventFilter, EventTx};

use super::PlatformError;

async fn handle_events(
    mut rx: UnboundedReceiver<wineventhook::WindowEvent>,
    event_tx: EventTx,
    event_filter: EventFilter,
) {
    while let Some(event) = rx.recv().await {
        if let Some(hwnd) = event.window_handle() {
            let Some(event) = super::event::make_event(event) else {
                continue;
            };

            if !event_filter.contains(&event) {
                continue;
            }

            let hwnd = Foundation::HWND(hwnd.as_ptr() as _);
            // SAFETY: `hwnd` is a valid window handle.
            let window = crate::Window(unsafe { super::PlatformWindow::new(hwnd) });

            if event_tx.send((window, event)).is_err() {
                break;
            };
        }
    }
}

pub async fn make_wineventhook_task(
    pid: i32,
    event_tx: EventTx,
    event_filter: EventFilter,
) -> Result<WindowEventHook, PlatformError> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let hook = WindowEventHook::hook(
        wineventhook::EventFilter::default()
            .events(raw_event::SYSTEM_START..raw_event::OBJECT_LOCATIONCHANGE)
            .process(std::num::NonZero::new(pid as _).unwrap()),
        tx,
    )
    .await?;

    tokio::task::spawn(async move {
        handle_events(rx, event_tx, event_filter).await;
    });

    Ok(hook)
}
