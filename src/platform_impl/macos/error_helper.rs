use accessibility_sys::AXError;

use crate::Error;

/// A trait to convert `AXError` into a `Result` type.
pub trait AXErrorIntoResult {
    /// Converts the `AXError` into a `Result`.
    ///
    /// # Parameters
    /// - `ok`: The value to return if the error indicates success.
    ///
    /// # Returns
    /// A `Result` containing the value or the `AXError`.
    fn into_result<T>(self, ok: T) -> Result<T, AXError>;
}

impl AXErrorIntoResult for AXError {
    fn into_result<T>(self, ok: T) -> Result<T, AXError> {
        if self == accessibility_sys::kAXErrorSuccess {
            Ok(ok)
        } else {
            Err(self)
        }
    }
}

impl From<AXError> for Error {
    /// Converts an `AXError` into a library-specific `Error`.
    fn from(error: AXError) -> Error {
        match error {
            accessibility_sys::kAXErrorAPIDisabled => Error::PermissinoDenied,
            another => Error::PlatformSpecificError(another),
        }
    }
}

impl From<accessibility::Error> for Error {
    /// Converts an `accessibility::Error` into a library-specific `Error`.
    fn from(error: accessibility::Error) -> Error {
        match error {
            accessibility::Error::Ax(raw) => Error::PlatformSpecificError(raw),
            // ↓ This error may indicate that the library is broken.
            another => panic!("Unexpected error is occurred: {another:?}"),
        }
    }
}
