use slotmap::DefaultKey;

use crate::{use_state_impl::LocationEngineGet, StorageKey};

/// An error that occurred while running an application.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The futures executor could not be created.
    #[error("the state engine is already mut,\nmsg:{1},\nat:{0}")]
    EngineAlreadyMut(LocationEngineGet, String),
    #[error("state store can't get DefaultKey use StorageKey : {0:?}")]
    StoreNoKey(StorageKey),
    #[error("state store can't get existing_secondary_map for Type : {0}")]
    StoreNoVarMapForType(&'static str),
    #[error("state store can't get DefaultKey use StorageKey :{0:?}, can't get existing_secondary_map for Type : {1}")]
    StoreNoKeyNoVarMapForType(StorageKey, &'static str),
    #[error("secondary_map (for k:DefaultKey v:Var) can't get var use key :{0:?}, key by StorageKey : {1:?}")]
    SecMapNoKey(DefaultKey, StorageKey),
    // ─────────────────────────────────────────────────────────────────────────────
    // #[error("...")]
    //     Other {
    //         #[backtrace]
    //         source: Box<dyn std::error::Error>,
    //     },
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
