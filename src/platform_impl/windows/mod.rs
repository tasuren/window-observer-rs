use helper::unhook_win_event;
use windows::Win32::{
    Foundation,
    UI::{Accessibility, WindowsAndMessaging},
};

use crate::{Error, Event, EventCallback};

mod callback;
mod helper;
mod window;

pub use window::Window;
pub use windows::core::Error as OSError;

pub struct WindowObserver {
    pid: i32,
    state: callback::State,
}

impl WindowObserver {
    pub fn new(pid: i32, callback: EventCallback) -> Result<Self, Error> {
        Ok(Self {
            pid,
            state: callback::State::Callback(callback::CallbackState {
                events: Default::default(),
                callback,
            }),
        })
    }

    pub fn is_running(&self) -> bool {
        self.state.is_running()
    }

    pub fn add_target_event(&mut self, target: Event) {
        self.state.add_target_event(target)
    }

    pub fn remove_target_event(&mut self, target: Event) -> Option<Event> {
        self.state.remove_target_event(target)
    }

    pub fn start(&mut self) -> Result<(), Error> {
        let hook = unsafe {
            Accessibility::SetWinEventHook(
                WindowsAndMessaging::EVENT_MIN,
                WindowsAndMessaging::EVENT_SYSTEM_END,
                Foundation::HMODULE(std::ptr::null_mut()),
                Some(callback::observer_callback),
                self.pid as _,
                0,
                WindowsAndMessaging::WINEVENT_OUTOFCONTEXT,
            )
            .0 as isize
        };
        self.state.insert_global_state(hook)
    }

    pub fn join(&self) {
        while self.is_running() {
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    }

    pub fn stop(&mut self) -> Result<(), Error> {
        unhook_win_event(self.state.remove_global_state()?).map_err(Into::into)
    }
}

impl Drop for WindowObserver {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
