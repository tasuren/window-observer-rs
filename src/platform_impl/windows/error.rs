#[derive(Debug, thiserror::Error)]
pub enum OSError {
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Windows error: {0}")]
    WinAPIError(#[from] windows::core::Error),
}
