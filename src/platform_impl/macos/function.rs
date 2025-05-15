use accessibility::AXUIElement;
use accessibility_sys::{AXError, AXValueRef};
use core_foundation::{
    base::{CFTypeRef, ToVoid},
    string::CFString,
};

use super::error_helper::AXErrorIntoResult;

pub fn ax_ui_element_copy_attribute_value(
    element: &AXUIElement,
    attribute: &str,
) -> Result<CFTypeRef, AXError> {
    let mut value: CFTypeRef = std::ptr::null();

    unsafe {
        accessibility_sys::AXUIElementCopyAttributeValue(
            element.to_void() as _,
            CFString::new(attribute).to_void() as _,
            &mut value,
        )
    }
    .into_result(value)
}

pub unsafe fn ax_value_get_value<T>(
    value: AXValueRef,
    r#type: accessibility_sys::AXValueType,
) -> Option<T> {
    let mut result = std::mem::MaybeUninit::<T>::uninit();

    unsafe {
        if accessibility_sys::AXValueGetValue(value, r#type, result.as_mut_ptr() as _) {
            Some(result.assume_init())
        } else {
            None
        }
    }
}

pub fn ax_is_process_trusted() -> bool {
    unsafe { accessibility_sys::AXIsProcessTrusted() }
}
