use emg_futures::futures;

/// An error that occurred while running an application.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The futures executor could not be created.
    #[error("the futures executor could not be created")]
    ExecutorCreationFailed(futures::io::Error),

    /// The application window could not be created.
    #[error("the application window could not be created")]
    WindowCreationFailed(Box<dyn std::error::Error + Send + Sync>),

    /// The application graphics context could not be created.
    #[error("the application graphics context could not be created")]
    GraphicsCreationFailed(emg_graphics_backend::Error),
}
// unsafe impl Send for Error {}
// unsafe impl Sync for Error {}

#[cfg(all(feature = "gpu"))]
impl From<emg_winit::Error> for Error {
    fn from(error: emg_winit::Error) -> Self {
        match error {
            emg_winit::Error::ExecutorCreationFailed(error) => Self::ExecutorCreationFailed(error),
            emg_winit::Error::WindowCreationFailed(error) => {
                Self::WindowCreationFailed(Box::new(error))
            }
            emg_winit::Error::GraphicsCreationFailed(error) => Self::GraphicsCreationFailed(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    const fn assert_send_sync() {
        #[allow(clippy::extra_unused_type_parameters)]
        const fn _assert<T: Send + Sync>() {}
        _assert::<Error>();
    }
}
