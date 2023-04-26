use std::{ops::Deref, panic::Location};

/*
 * @Author: Rais
 * @Date: 2023-03-28 17:20:34
 * @LastEditTime: 2023-04-24 17:39:39
 * @LastEditors: Rais
 * @Description:
 */
#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub struct TopoKey {
    // pub ctx: Option<SlottedKey>,
    pub id: topo::CallId,
}

impl TopoKey {
    #[must_use]
    pub const fn new(id: topo::CallId) -> Self {
        Self { id }
    }
}
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub enum StorageKey {
    // SlottedKey(SlottedKey),
    TopoKey(TopoKey),
}

impl StorageKey {
    #[must_use]
    pub const fn as_topo_key(&self) -> Option<&TopoKey> {
        match self {
            Self::TopoKey(v) => Some(v),
            _ => None,
        }
    }
}

impl From<TopoKey> for StorageKey {
    fn from(v: TopoKey) -> Self {
        Self::TopoKey(v)
    }
}
// impl From<&TopoKey> for StorageKey {
//     fn from(v: &TopoKey) -> Self {
//         Self::TopoKey(*v)
//     }
// }

// ─────────────────────────────────────────────────────────────────────────────

#[derive(Copy, Clone, Debug)]
pub struct LocationEngineGet(pub(crate) &'static Location<'static>);

impl LocationEngineGet {
    #[allow(clippy::new_without_default)]
    #[track_caller]
    #[must_use]
    pub(crate) fn new() -> Self {
        illicit::get::<Self>()
            .as_deref()
            .map_or_else(|_| Self(Location::caller()), |x| *x)
    }
    #[track_caller]
    #[must_use]
    fn reset_new() -> Self {
        Self(Location::caller())
    }
}

impl Deref for LocationEngineGet {
    type Target = &'static Location<'static>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
