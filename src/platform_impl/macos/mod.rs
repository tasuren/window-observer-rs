//! Bindings for macOS

use core_foundation::{
    base::{TCFType, ToVoid},
    runloop,
    string::{CFString, CFStringRef},
};

pub use accessibility_sys;
pub use core_foundation;
use helper::ax_ui_element_copy_attribute_value;

use crate::{Error, Event, EventCallback};

pub mod helper;
pub mod window;

pub use helper::OSError;
pub use window::Window;

pub struct WindowObserver {
    _pid: i32,
    element: accessibility_sys::AXUIElementRef,
    observer: *mut accessibility_sys::__AXObserver,
    pub callback: EventCallback,
}

extern "C" fn observer_callback(
    _observer: accessibility_sys::AXObserverRef,
    _element: accessibility_sys::AXUIElementRef,
    notification: CFStringRef,
    refcon: *mut std::ffi::c_void,
) {
    let (notification, refcon) = unsafe {
        (
            CFString::wrap_under_get_rule(notification).to_string(),
            &*(refcon as *const WindowObserver),
        )
    };

    // Convert the notification name to enum Event.
    let event = match notification.as_ref() {
        "AXMoved" => Event::Moved,
        "AXResized" => Event::Resized,
        "AXApplicationActivated" => Event::Activated,
        _ => {
            return;
        }
    };

    // Pick window.
    let window = window::Window(unsafe {
        ax_ui_element_copy_attribute_value(
            refcon.element,
            accessibility_sys::kAXFocusedWindowAttribute,
        )
        .unwrap()
    } as _);

    (refcon.callback)(event, crate::Window(window));
}

impl WindowObserver {
    pub fn new(pid: i32, callback: EventCallback) -> Result<Self, crate::Error> {
        unsafe {
            if !accessibility_sys::AXIsProcessTrusted() {
                return Err(crate::Error::PermissinoDenied);
            };
        }

        let mut observer = std::ptr::null_mut();
        unsafe {
            accessibility_sys::AXObserverCreate(pid, observer_callback, &mut observer);
        }

        Ok(Self {
            _pid: pid,
            element: unsafe { accessibility_sys::AXUIElementCreateApplication(pid) },
            callback,
            observer,
        })
    }

    pub fn add_target_event(&self, event: Event) {
        let notification = match event {
            Event::Activated => accessibility_sys::kAXApplicationActivatedNotification,
            Event::Moved => accessibility_sys::kAXMovedNotification,
            Event::Resized => accessibility_sys::kAXResizedNotification,
        };

        unsafe {
            accessibility_sys::AXObserverAddNotification(
                self.observer,
                self.element,
                CFString::new(notification).to_void() as _,
                self as *const Self as _,
            );
        }
    }

    pub fn start(&mut self) -> Result<(), Error> {
        unsafe {
            runloop::CFRunLoopAddSource(
                runloop::CFRunLoopGetCurrent(),
                accessibility_sys::AXObserverGetRunLoopSource(self.observer),
                runloop::kCFRunLoopDefaultMode,
            );
        };

        Ok(())
    }

    pub fn join(&self) {
        unsafe { core_foundation::runloop::CFRunLoopRun() };
    }

    pub fn stop(&self) -> () {
        if !self.observer.is_null() {
            unsafe {
                runloop::CFRunLoopRemoveSource(
                    runloop::CFRunLoopGetCurrent(),
                    accessibility_sys::AXObserverGetRunLoopSource(self.observer),
                    runloop::kCFRunLoopDefaultMode,
                );
            }
        }

        ()
    }
}

impl Drop for WindowObserver {
    fn drop(&mut self) {
        self.stop();
        unsafe {
            core_foundation::base::CFRelease(self.observer as _);
        }
    }
}
