//! Bindings for macOS

use std::{ffi::c_void, ptr::null_mut};

use accessibility_sys::{
    AXIsProcessTrusted, AXObserverAddNotification, AXObserverCreate, AXObserverGetRunLoopSource,
    AXObserverRef, AXUIElementCreateApplication, AXUIElementRef, __AXObserver,
    kAXApplicationActivatedNotification, kAXMovedNotification, kAXPositionAttribute,
    kAXResizedNotification, AXUIElementCopyAttributeValue, AXValueGetType, AXValueRef,
};
use core_foundation::{
    base::{CFRelease, TCFType, ToVoid},
    runloop::{
        kCFRunLoopDefaultMode, CFRunLoopAddSource, CFRunLoopGetCurrent, CFRunLoopRemoveSource,
        CFRunLoopRun,
    },
    string::{CFString, CFStringRef},
};

pub use accessibility_sys;
pub use core_foundation;

use crate::Event;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Permissions denied. I want the Accessibility Permission.")]
    PermissionsDenied,
    #[error("Error code: {0}")]
    SomethingWentWrong(i32),
}

pub struct WindowObserver {
    pid: i32,
    elem: AXUIElementRef,
    observer: *mut __AXObserver,
    pub callback: Box<dyn FnMut(Event)>,
}

extern "C" fn observer_callback(
    _observer: AXObserverRef,
    element: AXUIElementRef,
    notification: CFStringRef,
    refcon: *mut c_void,
) {
    let value: AXValueRef = null_mut();
    unsafe {
        AXUIElementCopyAttributeValue(
            element,
            CFString::new(kAXPositionAttribute).to_void() as _,
            value as _,
        );
        println!("{}", value.is_null());
        let type_ = AXValueGetType(value);
        //AXValueType
        println!("{}", type_);
    }

    // Convert the notification name to enum Event.
    let (notification, mut window_observer) = unsafe {
        (
            CFString::wrap_under_get_rule(notification).to_string(),
            Box::from_raw(refcon as *mut WindowObserver),
        )
    };
    let event = match notification.as_ref() {
        "AXMoved" => Event::Moved,
        "AXResized" => Event::Resized,
        "AXApplicationActivated" => Event::Activated,
        _ => {
            return;
        }
    };

    (window_observer.callback)(event);

    // Box will call the desctructor of WindowObserver
    // but the Rust Compiler compile to automatically call
    // the desctructor when ownership is dropped too.
    // To prevent Box from making dangling pointer,
    // let the Rust Compiler don't automatically drop it.
    std::mem::forget(window_observer);
}

impl WindowObserver {
    pub fn new(pid: i32, callback: Box<dyn FnMut(Event)>) -> Result<Self, Error> {
        unsafe {
            if !AXIsProcessTrusted() {
                return Err(Error::PermissionsDenied);
            };
        }

        let mut observer = null_mut();
        unsafe {
            let result = AXObserverCreate(pid, observer_callback, &mut observer);

            if result != accessibility_sys::kAXErrorSuccess {
                return Err(Error::SomethingWentWrong(result));
            }
        }

        Ok(Self {
            pid,
            elem: unsafe { AXUIElementCreateApplication(pid) },
            observer,
            callback,
        })
    }

    pub fn add_target_event(&self, event: Event) {
        let notification = match event {
            Event::Activated => kAXApplicationActivatedNotification,
            Event::Moved => kAXMovedNotification,
            Event::Resized => kAXResizedNotification,
        };

        unsafe {
            AXObserverAddNotification(
                self.observer,
                self.elem,
                CFString::new(notification).to_void() as _,
                self as *const Self as _,
            );
        }
    }

    pub fn start(&mut self) -> Result<(), Error> {
        unsafe {
            CFRunLoopAddSource(
                CFRunLoopGetCurrent(),
                AXObserverGetRunLoopSource(self.observer),
                kCFRunLoopDefaultMode,
            );
        };

        Ok(())
    }

    pub fn join(&self) {
        unsafe { CFRunLoopRun() };
    }

    pub fn stop(&self) -> () {
        if !self.observer.is_null() {
            unsafe {
                CFRunLoopRemoveSource(
                    CFRunLoopGetCurrent(),
                    AXObserverGetRunLoopSource(self.observer),
                    kCFRunLoopDefaultMode,
                );

                CFRelease(self.observer as _);
            }
        }

        ()
    }
}

impl Drop for WindowObserver {
    fn drop(&mut self) {
        self.stop();
    }
}
