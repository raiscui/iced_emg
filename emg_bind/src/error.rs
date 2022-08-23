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

#[cfg(all(feature = "gpu"))]
impl From<emg_winit::Error> for Error {
    fn from(error: emg_winit::Error) -> Error {
        match error {
            emg_winit::Error::ExecutorCreationFailed(error) => Error::ExecutorCreationFailed(error),
            emg_winit::Error::WindowCreationFailed(error) => {
                Error::WindowCreationFailed(Box::new(error))
            }
            emg_winit::Error::GraphicsCreationFailed(error) => Error::GraphicsCreationFailed(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assert_send_sync() {
        fn _assert<T: Send + Sync>() {}
        _assert::<Error>();
    }
}
