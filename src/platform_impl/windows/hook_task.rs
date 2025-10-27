use tokio::sync::mpsc::UnboundedReceiver;
use window_getter::platform_impl::get_window;
use windows::Win32::Foundation;
use wineventhook::{WindowEventHook, raw_event};

use super::{error::WindowsError, event_interpreter::EventInterpreter};
use crate::{EventFilter, EventTx};

fn handle_events(
    mut rx: UnboundedReceiver<wineventhook::WindowEvent>,
    mut event_interpreter: EventInterpreter,
) {
    while let Some(event) = rx.blocking_recv() {
        if let Some(hwnd) = event.window_handle() {
            let hwnd = Foundation::HWND(hwnd.as_ptr() as _);
            let Some(window) = get_window(hwnd).map(|w| w.into_platform_window()) else {
                // If hwnd is not valid window, continue;
                continue;
            };

            event_interpreter.interpret_wineventhook_event(window, event);
        }
    }
}

pub async fn make_wineventhook_task(
    pid: u32,
    event_tx: EventTx,
    event_filter: EventFilter,
) -> Result<WindowEventHook, WindowsError> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let hook = WindowEventHook::hook(
        wineventhook::EventFilter::default()
            .events(raw_event::SYSTEM_START..raw_event::OBJECT_LOCATIONCHANGE),
        tx,
    )
    .await?;

    std::thread::spawn(move || {
        let event_interpreter = EventInterpreter::new(pid, event_tx, event_filter);

        handle_events(rx, event_interpreter);
    });

    Ok(hook)
}
