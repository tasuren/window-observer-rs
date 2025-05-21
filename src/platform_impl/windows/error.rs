/// Represents errors that can occur in the Windows-specific implementation.
#[derive(Debug, thiserror::Error)]
pub enum OSError {
    /// An IO error occurred.
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    /// A Windows API error occurred.
    #[error("Windows error: {0}")]
    WinAPIError(#[from] windows::core::Error),
}
