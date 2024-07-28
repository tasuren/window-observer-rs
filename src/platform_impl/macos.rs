//! Bindings for macOS

use std::{ffi::c_void, ptr::null_mut};

use accessibility_sys::{
    AXIsProcessTrusted, AXObserverAddNotification, AXObserverCreate, AXObserverGetRunLoopSource,
    AXObserverRef, AXUIElementCreateApplication, AXUIElementRef, __AXObserver,
    kAXResizedNotification,
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
    pub callback: Box<dyn FnMut()>,
}

extern "C" fn observer_callback(
    _observer: AXObserverRef,
    _element: AXUIElementRef,
    notification: CFStringRef,
    refcon: *mut c_void,
) {
    let mut window_observer = unsafe {
        println!("{}", CFString::wrap_under_get_rule(notification));
        Box::from_raw(refcon as *mut WindowObserver)
    };
    (window_observer.callback)();
    // Box will call the desctructor of WindowObserver
    // but the Rust Compiler compile to automatically call
    // the desctructor when ownership is dropped too.
    // To prevent Box from making dangling pointer,
    // let the Rust Compiler don't automatically drop it.
    std::mem::forget(window_observer);
}

impl WindowObserver {
    pub fn new(pid: i32, callback: Box<dyn FnMut()>) -> Result<Self, Error> {
        unsafe {
            if !AXIsProcessTrusted() {
                return Err(Error::PermissionsDenied);
            };
        }

        Ok(Self {
            pid,
            elem: unsafe { AXUIElementCreateApplication(pid) },
            observer: null_mut(),
            callback,
        })
    }

    pub fn start(&mut self) -> Result<(), Error> {
        unsafe {
            let result = AXObserverCreate(self.pid, observer_callback, &mut self.observer);

            if result != accessibility_sys::kAXErrorSuccess {
                return Err(Error::SomethingWentWrong(result));
            }

            AXObserverAddNotification(
                self.observer,
                self.elem,
                CFString::new(kAXResizedNotification).to_void() as _,
                self as *const Self as _,
            );
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
