/// Represents errors that can occur in the Windows-specific implementation.
#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    /// An IO error occurred.
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    /// An error occurred from underlying implementation of [`window_getter`].
    #[error("Window getter error: {0}")]
    WindowGetterError(#[from] window_getter::platform_impl::PlatformError),
}
