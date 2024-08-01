use std::sync::LazyLock;

use windows::Win32::{Foundation, UI::Accessibility};

use crate::{Error, Event, EventCallback};

pub struct CallbackState {
    pub events: Vec<Event>,
    pub callback: EventCallback,
}

pub enum State {
    Hook(isize),
    Callback(CallbackState),
}

impl State {
    pub fn remove_global_state(&mut self) -> Result<isize, Error> {
        if let Self::Hook(hook) = *self {
            *self = Self::Callback(CALLBACK_STATES.write().unwrap().remove(&hook).unwrap());
            Ok(hook)
        } else {
            Err(Error::AlreadyStopped)
        }
    }

    pub fn insert_global_state(&mut self, hook: isize) -> Result<(), Error> {
        if let Self::Callback(state) = std::mem::replace(self, Self::Hook(hook)) {
            CALLBACK_STATES.write().unwrap().insert(hook, state);
            Ok(())
        } else {
            Err(Error::AlreadyStarted)
        }
    }

    pub fn add_target_event(&mut self, target: Event) {
        match self {
            Self::Callback(state) => {
                state.events.push(target);
            }
            Self::Hook(hook) => {
                CALLBACK_STATES
                    .write()
                    .unwrap()
                    .get_mut(&hook)
                    .unwrap()
                    .events
                    .push(target);
            }
        }
    }

    pub fn remove_target_event(&mut self, target: Event) -> Option<Event> {
        let rm = |state: &mut CallbackState| {
            if let Some(i) = state.events.iter().position(|x| x == &target) {
                Some(state.events.remove(i))
            } else {
                None
            }
        };

        match self {
            Self::Callback(state) => rm(state),
            Self::Hook(hook) => {
                let mut states = CALLBACK_STATES.write().unwrap();
                rm(states.get_mut(&hook).unwrap())
            }
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self, Self::Hook(_))
    }
}

static CALLBACK_STATES: LazyLock<
    std::sync::RwLock<std::collections::HashMap<isize, CallbackState>>,
> = LazyLock::new(Default::default);

pub extern "system" fn observer_callback(
    hwineventhook: Accessibility::HWINEVENTHOOK,
    event: u32,
    hwnd: Foundation::HWND,
    _idobject: i32,
    _idchild: i32,
    _ideventthread: u32,
    _dwmseventtime: u32,
) {
    if let Some(state) = CALLBACK_STATES.read().unwrap().get(&(hwineventhook.0 as _)) {
        for event in super::helper::raw_to_event(event) {
            (state.callback)(event, crate::Window(super::window::Window(hwnd)))
        }
    };
}
