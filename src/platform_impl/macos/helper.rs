use accessibility_sys::AXValueRef;
use core_foundation::{
    base::{CFTypeRef, ToVoid},
    string::CFString,
};

use crate::{Error, Event};

#[derive(Debug, thiserror::Error)]
pub enum OSError {
    #[error("Assistive applications are not enabled in System Preferences.")]
    APIDisabled(i32),
    #[error("The referenced action is not supported. Alternatively, you can return the eventNotHandledErr error.")]
    ActionUnsupported(i32),
    #[error("The referenced attribute is not supported. Alternatively, you can return the eventNotHandledErr error.")]
    AttributeUnsupported(i32),
    #[error(
        "A fundamental error has occurred, such as a failure to allocate memory during processing."
    )]
    CannotComplete(i32),
    #[error("A system error occurred, such as the failure to allocate an object.")]
    Failure(i32),
    #[error("The value received in this event is an invalid value for this attribute. This also applies for invalid parameters in parameterized attributes.")]
    IllegalArgument(i32),
    #[error("The accessibility object received in this event is invalid.")]
    InvalidUIElement(i32),
    #[error("The observer for the accessibility object received in this event is invalid.")]
    InvalidUIElementObserver(i32),
    #[error("The requested value or AXUIElementRef does not exist.")]
    NoValue(i32),
    #[error("Not enough precision.")]
    NotEnoughPrecision(i32),
    #[error("Indicates that the function or method is not implemented (this can be returned if a process does not support the accessibility API).")]
    NotImplemented(i32),
    #[error("This notification has already been registered for.")]
    NotificationAlreadyRegistered(i32),
    #[error("Indicates that a notification is not registered yet.")]
    NotificationNotRegistered(i32),
    #[error("The notification is not supported by the AXUIElementRef.")]
    NotificationUnsupported(i32),
    #[error("The parameterized attribute is not supported. Alternatively, you can return the eventNotHandledErr error.")]
    ParameterizedAttributeUnsupported(i32),
}

impl OSError {
    pub fn from_raw(value: i32) -> Option<Self> {
        if value == accessibility_sys::kAXErrorSuccess {
            return None;
        };

        Some(match value {
            accessibility_sys::kAXErrorAPIDisabled => Self::APIDisabled(value),
            accessibility_sys::kAXErrorActionUnsupported => Self::ActionUnsupported(value),
            accessibility_sys::kAXErrorAttributeUnsupported => Self::AttributeUnsupported(value),
            accessibility_sys::kAXErrorCannotComplete => Self::CannotComplete(value),
            accessibility_sys::kAXErrorFailure => Self::Failure(value),
            accessibility_sys::kAXErrorIllegalArgument => Self::IllegalArgument(value),
            accessibility_sys::kAXErrorInvalidUIElement => Self::InvalidUIElement(value),
            accessibility_sys::kAXErrorInvalidUIElementObserver => {
                Self::InvalidUIElementObserver(value)
            }
            accessibility_sys::kAXErrorNoValue => Self::NoValue(value),
            accessibility_sys::kAXErrorNotEnoughPrecision => Self::NotEnoughPrecision(value),
            accessibility_sys::kAXErrorNotImplemented => Self::NotImplemented(value),
            accessibility_sys::kAXErrorNotificationAlreadyRegistered => {
                Self::NotificationAlreadyRegistered(value)
            }
            accessibility_sys::kAXErrorNotificationNotRegistered => {
                Self::NotificationNotRegistered(value)
            }
            accessibility_sys::kAXErrorNotificationUnsupported => {
                Self::NotificationUnsupported(value)
            }
            accessibility_sys::kAXErrorParameterizedAttributeUnsupported => {
                Self::ParameterizedAttributeUnsupported(value)
            }
            _ => unreachable!(),
        })
    }
}

impl From<OSError> for Error {
    fn from(error: OSError) -> Error {
        match error {
            OSError::APIDisabled(_) => Error::PermissinoDenied,
            another => Error::PlatformSpecificError(another),
        }
    }
}

pub(crate) fn event_to_raw<'a>(event: Event) -> &'a str {
    match event {
        Event::Activated => accessibility_sys::kAXApplicationActivatedNotification,
        Event::Moved => accessibility_sys::kAXMovedNotification,
        Event::Resized => accessibility_sys::kAXResizedNotification,
    }
}

pub(crate) unsafe fn ax_ui_element_copy_attribute_value(
    element: accessibility_sys::AXUIElementRef,
    attribute: &str,
) -> Result<CFTypeRef, OSError> {
    let mut value: CFTypeRef = std::ptr::null();

    let result = accessibility_sys::AXUIElementCopyAttributeValue(
        element,
        CFString::new(attribute).to_void() as _,
        &mut value,
    );
    if let Some(error) = OSError::from_raw(result) {
        return Err(error);
    };

    Ok(value)
}

pub(crate) unsafe fn ax_value_get_value<T>(
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
