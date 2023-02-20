/*
 * @Author: Rais
 * @Date: 2022-08-11 18:22:19
 * @LastEditTime: 2023-01-13 12:16:14
 * @LastEditors: Rais
 * @Description:
 */
//! Handle events of a user interface.

use crate::keyboard;
use crate::mouse;
use crate::touch;
use crate::window;
use bitflags::bitflags;

pub type EventWithFlagType = ((EventFlag, u32), Event);

// Event bigflags
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct EventFlag: u32 {
        const KEYBOARD =            1<<0;
        const MOUSE =               1<<1;
        const WINDOW =              1<<2;
        const TOUCH =               1<<3;
        const PLATFORM_SPECIFIC =   1<<4;

    }
}

/// A user interface event.
///
/// _**Note:** This type is largely incomplete! If you need to track
/// additional events, feel free to [open an issue] and share your use case!_
///
/// [open an issue]: https://github.com/iced-rs/iced/issues
#[derive(Debug, Clone, PartialEq)]
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
}

impl Event {
    // pub fn to_str(&self) -> IdStr {
    //     match self {
    //         Event::Keyboard(_) => todo!(),
    //         Event::Mouse(x) => match x {
    //             emg_common::mouse::Event::ButtonReleased(_) => IdStr::new_inline("click"),
    //             other => other.to_compact_string(),
    //         },
    //         Event::Window(_) => IdStr::new_inline("Window"), //TODO  make it right
    //         Event::Touch(_) => todo!(),
    //         Event::PlatformSpecific(_) => todo!(),
    //     }
    // }
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
