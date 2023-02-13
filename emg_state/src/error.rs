use crate::use_state_impl::LocationEngineGet;

/// An error that occurred while running an application.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The futures executor could not be created.
    #[error("the state engine is already mut,\nmsg:{1},\nat:{0}")]
    EngineAlreadyMut(LocationEngineGet, String),
}
