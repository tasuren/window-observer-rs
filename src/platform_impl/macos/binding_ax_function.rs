//! This module provides bindings for accessibility functions
//! related to [`AXUIElement`] and [`AXValueRef`].

use accessibility::AXUIElement;
use accessibility_sys::{AXError, AXValueGetValue, AXValueRef};
use core_foundation::{
    base::{CFTypeRef, TCFType},
    boolean::CFBoolean,
    dictionary::CFDictionary,
    string::CFString,
};

use super::error::AXErrorIntoResult;

/// Copies the value of a specified attribute from an [`AXUIElement`].
///
/// # Parameters
/// - `element`: The `AXUIElement` to query.
/// - `attribute`: The name of the attribute to retrieve.
///
/// # Returns
/// A [`Result`] containing the attribute value or an [`AXError`].
pub fn ax_ui_element_copy_attribute_value(
    element: &AXUIElement,
    attribute: &str,
) -> Result<CFTypeRef, AXError> {
    let mut value: CFTypeRef = std::ptr::null();

    unsafe {
        accessibility_sys::AXUIElementCopyAttributeValue(
            element.as_concrete_TypeRef(),
            CFString::new(attribute).as_concrete_TypeRef(),
            &mut value,
        )
    }
    .into_result(value)
}

/// Utility function for [`AXValueGetValue`].
///
/// # Safety
/// The `value` must be the pointer of [`AXValue`][ax_value].
///
/// # Parameters
/// - `value`: The AXValue to extract.
/// - `type`: The expected type of the value.
///
/// # Returns
/// An [`Option`] containing the extracted value if successful.
///
/// [ax_value]: https://developer.apple.com/documentation/applicationservices/axvalue_h?language=objc
pub unsafe fn ax_value_get_value<T>(
    value: AXValueRef,
    r#type: accessibility_sys::AXValueType,
) -> Option<T> {
    let mut result = std::mem::MaybeUninit::<T>::uninit();

    unsafe {
        if AXValueGetValue(value, r#type, result.as_mut_ptr() as _) {
            Some(result.assume_init())
        } else {
            None
        }
    }
}

/// Checks if the current process is trusted for accessibility features.
pub fn ax_is_process_trusted() -> bool {
    unsafe { accessibility_sys::AXIsProcessTrusted() }
}

/// Checks if the current process is trusted for accessibility features.
///
/// # Parameters
/// - `prompt`: It indicates whether the user will be informed if the current process is untrusted.
///     Prompting occurs asynchronously and does not affect the return value.
pub fn ax_is_process_trusted_with_options(prompt: bool) -> bool {
    unsafe {
        let key = CFString::wrap_under_get_rule(accessibility_sys::kAXTrustedCheckOptionPrompt);
        let value = CFBoolean::from(prompt);
        let options = CFDictionary::from_CFType_pairs(&[(key, value)]);

        accessibility_sys::AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef())
    }
}

#[cfg(feature = "macos-private-api")]
pub fn ax_ui_element_get_window_id(element: &AXUIElement) -> Result<u32, super::error::MacOSError> {
    use accessibility_sys::{AXError, AXUIElementRef};

    use super::error::AXErrorIntoResult;

    unsafe extern "C" {
        fn _AXUIElementGetWindow(element: AXUIElementRef, out: *mut u32) -> AXError;
    }

    unsafe {
        let mut out = 0;

        _AXUIElementGetWindow(element.as_concrete_TypeRef(), &mut out)
            .into_result(out)
            .map_err(super::error::MacOSError::Ax)
    }
}
