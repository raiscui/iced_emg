use super::{KeyCode, Modifiers};
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct EventFlag: u32 {
        const KEY_PRESSED =           1<<0;
        const KEY_RELEASED =            1<<1;
        const CHARACTER_RECEIVED =           1<<2;
        const MODIFIERS_CHANGED =         1<<3 ;

    }
}

/// A keyboard event.
///
/// _**Note:** This type is largely incomplete! If you need to track
/// additional events, feel free to [open an issue] and share your use case!_
///
/// [open an issue]: https://github.com/iced-rs/iced/issues
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// A keyboard key was pressed.
    KeyPressed {
        /// The key identifier
        key_code: KeyCode,

        /// The state of the modifier keys
        modifiers: Modifiers,
    },

    /// A keyboard key was released.
    KeyReleased {
        /// The key identifier
        key_code: KeyCode,

        /// The state of the modifier keys
        modifiers: Modifiers,
    },

    /// A unicode character was received.
    CharacterReceived(char),

    /// The keyboard modifiers have changed.
    ModifiersChanged(Modifiers),
}
