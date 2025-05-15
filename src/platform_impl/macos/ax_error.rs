pub use accessibility_sys::AXError;

use crate::Error;

pub enum AXErrorConvertError {
    AXErrorSuccess,
    UnknownError,
}

impl From<AXError> for Error {
    fn from(error: AXError) -> Error {
        match error {
            accessibility_sys::kAXErrorAPIDisabled => Error::PermissinoDenied,
            another => Error::PlatformSpecificError(another),
        }
    }
}

impl From<accessibility::Error> for Error {
    fn from(error: accessibility::Error) -> Error {
        match error {
            accessibility::Error::Ax(raw) => Error::PlatformSpecificError(raw),
            // ↓ This error may indicate that the library is broken.
            another => panic!("Unexpected error is occurred: {another:?}"),
        }
    }
}
