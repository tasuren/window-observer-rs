//! Bindings for macOS

use core_foundation::{
    base::{TCFType, ToVoid},
    runloop,
    string::{CFString, CFStringRef},
};

pub use accessibility_sys;
pub use core_foundation;
use helper::{ax_ui_element_copy_attribute_value, event_to_raw};

use crate::{Error, Event, EventCallback};

pub mod helper;
pub mod window;

pub use helper::OSError;
pub use window::Window;

/* A structure that groups objects whose addresses should remain unchanged. */
struct Controller {
    element: accessibility_sys::AXUIElementRef,
    pub callback: EventCallback,
}

pub struct WindowObserver {
    _pid: i32,
    observer: *mut accessibility_sys::__AXObserver,
    controller: Box<Controller>,
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
            &*(refcon as *mut Controller),
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

        let controller = Controller {
            element: unsafe { accessibility_sys::AXUIElementCreateApplication(pid) },
            callback,
        };

        Ok(Self {
            _pid: pid,
            controller: Box::new(controller),
            observer,
        })
    }

    pub fn add_target_event(&self, event: Event) -> Result<(), Error> {
        unsafe {
            accessibility_sys::AXObserverAddNotification(
                self.observer,
                self.controller.element,
                CFString::new(event_to_raw(event)).to_void() as _,
                &*self.controller as *const _ as _,
            );
        };

        Ok(())
    }

    pub fn remove_target_event(&self, event: Event) -> Result<(), Error> {
        unsafe {
            accessibility_sys::AXObserverRemoveNotification(
                self.observer,
                self.controller.element,
                CFString::new(event_to_raw(event)).to_void() as _,
            );
        };

        Ok(())
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

    pub fn stop(&self) -> Result<(), Error> {
        if !self.observer.is_null() {
            unsafe {
                runloop::CFRunLoopRemoveSource(
                    runloop::CFRunLoopGetCurrent(),
                    accessibility_sys::AXObserverGetRunLoopSource(self.observer),
                    runloop::kCFRunLoopDefaultMode,
                );
            }
        }

        Ok(())
    }
}

impl Drop for WindowObserver {
    fn drop(&mut self) {
        self.stop().unwrap();
        unsafe {
            core_foundation::base::CFRelease(self.observer as _);
        }
    }
}
