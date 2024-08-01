use std::{
    sync::{Arc, RwLock},
    thread,
};

use pollster::FutureExt as _;
use windows::Win32::Foundation;
use wineventhook::{EventFilter, WindowEventHook};

use crate::{Error, Event, EventCallback};

mod helper;
mod window;

pub use window::Window;
pub use windows::core::Error as OSError;

pub struct Handle {
    hook: WindowEventHook,
    thread: Option<thread::JoinHandle<()>>,
}

pub struct State {
    callback: EventCallback,
    events: Vec<Event>,
}

pub struct WindowObserver {
    pid: i32,
    state: Arc<RwLock<State>>,
    handle: Option<Handle>,
}

impl WindowObserver {
    pub fn new(pid: i32, callback: EventCallback) -> Result<Self, Error> {
        Ok(Self {
            pid,
            state: Arc::new(RwLock::new(State {
                callback,
                events: Vec::new(),
            })),
            handle: None,
        })
    }

    pub fn is_running(&self) -> bool {
        self.handle.is_some()
    }

    pub fn add_target_event(&mut self, event: Event) -> Result<(), Error> {
        self.state.write().unwrap().events.push(event);
        Ok(())
    }

    pub fn remove_target_event(&mut self, event: Event) -> Result<(), Error> {
        let mut state = self.state.write().unwrap();

        if let Some(i) = state.events.iter().position(|x| x == &event) {
            state.events.remove(i);
        };
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), Error> {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let hook = WindowEventHook::hook(
            EventFilter::default()
                .events(wineventhook::raw_event::all_system())
                .process(std::num::NonZero::new(self.pid as _).unwrap()),
            tx,
        )
        .block_on()
        .unwrap();

        let state = Arc::clone(&self.state);
        let handler = move || {
            while let Some(event) = rx.blocking_recv() {
                if let Some(hwnd) = event.window_handle() {
                    let state = state.read().unwrap();

                    for event in helper::make_event(event) {
                        if !state.events.contains(&event) {
                            continue;
                        }

                        (state.callback)(
                            event,
                            crate::Window(window::Window(Foundation::HWND(hwnd.as_ptr() as _))),
                        );
                    }
                }
            }
        };

        self.handle = Some(Handle {
            hook,
            thread: Some(thread::spawn(handler)),
        });
        Ok(())
    }

    pub fn join(&mut self) {
        if let Some(ref mut handle) = self.handle {
            handle.thread.take().unwrap().join().unwrap();
        };
    }

    pub fn stop(&mut self) -> Result<(), Error> {
        if let Some(handle) = self.handle.take() {
            handle.hook.unhook().block_on().unwrap();
            Ok(())
        } else {
            Err(Error::AlreadyStopped)
        }
    }
}

impl Drop for WindowObserver {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
