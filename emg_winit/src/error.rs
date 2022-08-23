use emg_futures::futures;

/// An error that occurred while running an application.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The futures executor could not be created.
    #[error("the futures executor could not be created")]
    ExecutorCreationFailed(futures::io::Error),

    /// The application window could not be created.
    #[error("the application window could not be created")]
    WindowCreationFailed(winit::error::OsError),

    /// The application graphics context could not be created.
    #[error("the application graphics context could not be created")]
    GraphicsCreationFailed(emg_graphics_backend::Error),
}

impl From<emg_graphics_backend::Error> for Error {
    fn from(error: emg_graphics_backend::Error) -> Error {
        Error::GraphicsCreationFailed(error)
    }
}
