use tokio::sync::mpsc::UnboundedReceiver;
use window_getter::platform_impl::get_window;
use windows::Win32::Foundation;
use wineventhook::{raw_event, WindowEventHook};

use super::{event::EventManager, PlatformError};
use crate::{EventFilter, EventTx};

fn handle_events(
    pid: u32,
    mut rx: UnboundedReceiver<wineventhook::WindowEvent>,
    event_tx: EventTx,
    event_filter: EventFilter,
) {
    let mut event_manager = EventManager::new(pid);

    while let Some(event) = rx.blocking_recv() {
        if let Some(hwnd) = event.window_handle() {
            let hwnd = Foundation::HWND(hwnd.as_ptr() as _);
            let Some(window) = get_window(hwnd).map(|w| w.into_inner()) else {
                continue;
            };

            let Some((event, window)) = event_manager.convert_event(window, event) else {
                continue;
            };

            if !event_filter.contains(&event) {
                continue;
            }

            let window = crate::Window(window);
            if event_tx.send((window, event)).is_err() {
                break;
            };
        }
    }
}

pub async fn make_wineventhook_task(
    pid: u32,
    event_tx: EventTx,
    event_filter: EventFilter,
) -> Result<WindowEventHook, PlatformError> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let hook = WindowEventHook::hook(
        wineventhook::EventFilter::default()
            .events(raw_event::SYSTEM_START..raw_event::OBJECT_LOCATIONCHANGE),
        tx,
    )
    .await?;

    std::thread::spawn(move || handle_events(pid, rx, event_tx, event_filter));

    Ok(hook)
}
