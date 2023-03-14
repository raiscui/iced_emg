/*
 * @Author: Rais
 * @Date: 2022-08-11 18:22:19
 * @LastEditTime: 2023-03-14 17:54:29
 * @LastEditors: Rais
 * @Description:
 */
//! Handle events of a user interface.
mod ev_identify;
use crate::drag;
use crate::keyboard;
use crate::mouse;
use crate::touch;
use crate::window;
use bitflags::bitflags;
pub use ev_identify::*;

///u32 是 二级 事件 flag
pub type EventWithFlagType = (EventIdentify, Event);

// Event bigflags
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct EventFlag: u32 {
        const KEYBOARD =            1<<0;
        const MOUSE =               1<<1;
        const WINDOW =              1<<2;
        const TOUCH =               1<<3;
        const PLATFORM_SPECIFIC =   1<<4;
        const DND =                1<<5;//Drag and Drop

    }
}

/// A user interface event.
///
/// _**Note:** This type is largely incomplete! If you need to track
/// additional events, feel free to [open an issue] and share your use case!_
///
/// [open an issue]: https://github.com/iced-rs/iced/issues
#[derive(Debug, Clone, PartialEq)]
//TODO global refpool
pub enum Event {
    /// A keyboard event
    Keyboard(keyboard::Event),

    /// A mouse event
    Mouse(mouse::Event),

    /// A window event
    Window(window::Event),

    /// A touch event
    Touch(touch::Event),

    /// A platform specific event
    PlatformSpecific(PlatformSpecific),

    /// A drag event
    DragDrop(drag::Event),
}

/// A platform specific event
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformSpecific {
    /// A MacOS specific event
    MacOS(MacOS),
}

/// Describes an event specific to MacOS
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MacOS {
    /// Triggered when the app receives an URL from the system
    ///
    /// _**Note:** For this event to be triggered, the executable needs to be properly [bundled]!_
    ///
    /// [bundled]: https://developer.apple.com/library/archive/documentation/CoreFoundation/Conceptual/CFBundles/BundleTypes/BundleTypes.html#//apple_ref/doc/uid/10000123i-CH101-SW19
    ReceivedUrl(String),
}
