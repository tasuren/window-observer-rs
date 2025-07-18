pub use accessibility::Error as PlatformError;
use accessibility_sys::AXError;

#[derive(Debug, thiserror::Error)]
#[error("Unexpected UI element is received: expected = {expected}, received = {received:?}")]
pub struct UnexpectedUIElementError {
    pub element: accessibility::AXUIElement,
    pub expected: &'static str,
    pub received: Result<String, accessibility::Error>,
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum MacOSError {
    #[error(transparent)]
    UnexpectedUIElement(#[from] UnexpectedUIElementError),
    #[error("Accessibility library error occurred")]
    Accessibility(#[from] accessibility::Error),
}

/// A trait to convert [`AXError`] into a [`Result`] type.
pub trait AXErrorIntoResult {
    /// Converts the [`AXError`] into a [`Result`].
    ///
    /// # Parameters
    /// - `ok`: The value to return if the error indicates success.
    ///
    /// # Returns
    /// A [`Result`] containing the value or the [`AXError`].
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
