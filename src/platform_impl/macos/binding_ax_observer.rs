//! This module provides bindings for [`AXObserverRef`].

use std::{ptr::NonNull, sync::Mutex};

use accessibility::AXUIElement;
use accessibility_sys::{pid_t, AXError, AXObserverRef};
use core_foundation::base::{FromVoid, TCFType};
use objc2_core_foundation::{CFRetained, CFRunLoopSource, CFString};

use super::error::AXErrorIntoResult;

/// A type alias for the callback function used by the [`AXObserver`].
/// It takes an [`AXUIElement`] and a [notification][notification] string as parameters.
///
/// [notification]: https://developer.apple.com/documentation/applicationservices/axnotificationconstants_h?language=objc
pub type ObserverCallback = Box<dyn FnMut(AXUIElement, String)>;

struct RefCon {
    callback: ObserverCallback,
}

extern "C" fn observer_callback(
    _observer: AXObserverRef,
    element: accessibility_sys::AXUIElementRef,
    notification: core_foundation::string::CFStringRef,
    refcon: *mut std::ffi::c_void,
) {
    let notification = unsafe { &*(notification as *const CFString) };
    let refcon = unsafe { &*(refcon as *mut Mutex<RefCon>) };
    let element = unsafe { AXUIElement::from_void(element as _) }.clone();

    if let Ok(mut refcon) = refcon.lock() {
        (refcon.callback)(element, notification.to_string());
    };
}

/// Represents an `AXObserver`, which observes accessibility notifications.
/// This is a wrapper around the [`AXObserverRef`].
pub struct AXObserver {
    raw: AXObserverRef,
    refcon: Box<Mutex<RefCon>>,
}

impl AXObserver {
    /// Creates a new `AXObserver` for a given process ID and callback function.
    /// The `AXObserver` will call the callback function when a notification is received.
    pub fn create(pid: pid_t, callback: ObserverCallback) -> Result<Self, AXError> {
        let mut observer: AXObserverRef = std::ptr::null_mut();

        unsafe {
            accessibility_sys::AXObserverCreate(pid, observer_callback, &mut observer)
                .into_result(())?;
        };

        Ok(Self {
            raw: observer,
            refcon: Box::new(Mutex::new(RefCon { callback })),
        })
    }

    /// Retrieves the [`AXObserverRef`] being used by this.
    pub fn raw(&self) -> AXObserverRef {
        self.raw
    }

    /// Adds a notification to be observed for a specific [`AXUIElement`].
    pub fn add_notification(
        &self,
        element: &AXUIElement,
        notification: &str,
    ) -> Result<(), AXError> {
        unsafe {
            accessibility_sys::AXObserverAddNotification(
                self.raw,
                element.as_concrete_TypeRef(),
                CFString::from_str(notification)
                    .downcast_ref::<CFString>()
                    .unwrap() as *const CFString as _,
                self.refcon.as_ref() as *const Mutex<RefCon> as _,
            )
        }
        .into_result(())
    }

    /// Removes a notification from being observed for a specific [`AXUIElement`].
    pub fn remove_notification(
        &self,
        element: &AXUIElement,
        notification: &str,
    ) -> Result<(), AXError> {
        unsafe {
            accessibility_sys::AXObserverRemoveNotification(
                self.raw,
                element.as_concrete_TypeRef(),
                CFString::from_str(notification)
                    .downcast_ref::<CFString>()
                    .unwrap() as *const CFString as _,
            )
        }
        .into_result(())
    }

    /// Retrieves the [`CFRunLoopSource`] associated with the `AXObserver`.
    pub fn get_run_loop_source(&self) -> CFRetained<CFRunLoopSource> {
        unsafe {
            let ptr = accessibility_sys::AXObserverGetRunLoopSource(self.raw);
            let ptr = NonNull::new_unchecked(ptr as *mut CFRunLoopSource);

            CFRetained::retain(ptr)
        }
    }
}

impl Drop for AXObserver {
    /// Releases the [`AXObserverRef`] when it is dropped.
    fn drop(&mut self) {
        unsafe {
            core_foundation::base::CFRelease(self.raw as _);
        }
    }
}
