use accessibility::AXUIElement;
use accessibility_sys::{AXError, AXValueRef};
use core_foundation::{
    base::{CFTypeRef, ToVoid},
    string::CFString,
};

use super::error_helper::AXErrorIntoResult;

/// Copies the value of a specified attribute from an `AXUIElement`.
///
/// # Parameters
/// - `element`: The `AXUIElement` to query.
/// - `attribute`: The name of the attribute to retrieve.
///
/// # Returns
/// A `Result` containing the attribute value or an `AXError`.
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

/// Utility function for `AXValueGetValue`.
///
/// # Safety
/// The `value` must be `AXValue`.
///
/// # Parameters
/// - `value`: The AXValue to extract.
/// - `type`: The expected type of the value.
///
/// # Returns
/// An `Option` containing the extracted value if successful.
pub unsafe fn ax_value_get_value<T>(
    value: AXValueRef,
    r#type: accessibility_sys::AXValueType,
) -> Option<T> {
    let mut result = std::mem::MaybeUninit::<T>::uninit();

    if accessibility_sys::AXValueGetValue(value, r#type, result.as_mut_ptr() as _) {
        Some(result.assume_init())
    } else {
        None
    }
}

/// Checks if the current process is trusted for accessibility features.
pub fn ax_is_process_trusted() -> bool {
    unsafe { accessibility_sys::AXIsProcessTrusted() }
}
