use std::{ptr::NonNull, sync::Mutex};

use accessibility::AXUIElement;
use accessibility_sys::{pid_t, AXError, AXObserverRef};
use core_foundation::base::TCFType;
use objc2_core_foundation::{CFRetained, CFRunLoopSource, CFString};

use super::error_helper::AXErrorIntoResult;

type Callback = Box<dyn Fn(String)>;

pub struct RefCon {
    callback: Callback,
}

extern "C" fn observer_callback(
    _observer: AXObserverRef,
    _element: accessibility_sys::AXUIElementRef,
    notification: core_foundation::string::CFStringRef,
    refcon: *mut std::ffi::c_void,
) {
    let notification = unsafe { &*(notification as *const CFString) };
    let refcon = unsafe { &*(refcon as *mut Mutex<RefCon>) };

    if let Ok(refcon) = refcon.lock() {
        (refcon.callback)(notification.to_string());
    };
}

pub struct AXObserver {
    pub raw: AXObserverRef,
    refcon: Box<Mutex<RefCon>>,
}

impl AXObserver {
    pub fn create(pid: pid_t, callback: Callback) -> Self {
        let mut observer: AXObserverRef = std::ptr::null_mut();

        unsafe {
            accessibility_sys::AXObserverCreate(pid, observer_callback, &mut observer);
        };

        Self {
            raw: observer,
            refcon: Box::new(Mutex::new(RefCon { callback })),
        }
    }

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

    pub fn get_run_loop_source(&self) -> CFRetained<CFRunLoopSource> {
        unsafe {
            let ptr = accessibility_sys::AXObserverGetRunLoopSource(self.raw);
            let ptr = NonNull::new_unchecked(ptr as *mut CFRunLoopSource);

            CFRetained::retain(ptr)
        }
    }
}

impl Drop for AXObserver {
    fn drop(&mut self) {
        unsafe {
            core_foundation::base::CFRelease(self.raw as _);
        }
    }
}
