use crate::Error;

#[derive(Debug, thiserror::Error)]
pub enum OSError {
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Windows error: {0}")]
    WindowsError(#[from] windows::core::Error),
}

impl From<OSError> for Error {
    fn from(err: OSError) -> Self {
        Error::PlatformSpecificError(err)
    }
}
