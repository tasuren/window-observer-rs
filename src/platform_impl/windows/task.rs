use tokio::sync::mpsc::UnboundedReceiver;
use windows::Win32::Foundation;
use wineventhook::WindowEventHook;

use crate::{EventFilter, EventTx};

use super::OSError;

async fn handle_events(
    mut rx: UnboundedReceiver<wineventhook::WindowEvent>,
    event_tx: EventTx,
    event_filter: EventFilter,
) {
    while let Some(event) = rx.recv().await {
        if let Some(hwnd) = event.window_handle() {
            for event in super::helper::make_event(event) {
                if !event_filter.contains(&event) {
                    continue;
                }

                let hwnd = Foundation::HWND(hwnd.as_ptr() as _);
                let window = crate::Window(super::window::WindowsWindow::new(hwnd));

                if event_tx.send((window, event)).is_err() {
                    break;
                };
            }
        }
    }
}

pub async fn make_wineventhook_task(
    pid: i32,
    event_tx: EventTx,
    event_filter: EventFilter,
) -> Result<WindowEventHook, OSError> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let hook = WindowEventHook::hook(
        wineventhook::EventFilter::default()
            .events(wineventhook::raw_event::all_system())
            .process(std::num::NonZero::new(pid as _).unwrap()),
        tx,
    )
    .await?;

    tokio::task::spawn(async move {
        handle_events(rx, event_tx, event_filter).await;
    });

    Ok(hook)
}
