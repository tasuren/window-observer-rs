use std::mem::MaybeUninit;

use smallvec::{smallvec, SmallVec};
use windows::Win32::{
    Foundation,
    UI::{Accessibility, WindowsAndMessaging},
};

use super::OSError;
use crate::Event;

pub fn raw_to_event(raw: u32) -> SmallVec<[Event; 2]> {
    match raw {
        WindowsAndMessaging::EVENT_SYSTEM_MOVESIZEEND => smallvec![Event::Moved, Event::Resized],
        WindowsAndMessaging::EVENT_SYSTEM_FOREGROUND => smallvec![Event::Activated],
        _ => SmallVec::default(),
    }
}

pub fn unhook_win_event(hook: isize) -> Result<(), OSError> {
    unsafe { Accessibility::UnhookWinEvent(Accessibility::HWINEVENTHOOK(hook as _)) }.ok()
}

pub fn get_window_rect(hwnd: Foundation::HWND) -> Result<Foundation::RECT, windows::core::Error> {
    let mut value = MaybeUninit::uninit();
    unsafe {
        WindowsAndMessaging::GetWindowRect(hwnd, value.as_mut_ptr())?;
        Ok(value.assume_init())
    }
}
