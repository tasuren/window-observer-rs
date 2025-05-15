use accessibility_sys::AXError;

pub trait AXErrorIntoResult {
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
