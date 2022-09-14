use crate::Pos;

// use crate::Point;
use super::Button;
use derive_more::Display;
/// A mouse event.
///
/// _**Note:** This type is largely incomplete! If you need to track
/// additional events, feel free to [open an issue] and share your use case!_
///
/// [open an issue]: https://github.com/iced-rs/iced/issues
#[derive(Debug, Clone, Copy, Display, PartialEq)]
pub enum Event {
    /// The mouse cursor entered the window.
    #[display(fmt = "CursorEntered")]
    CursorEntered,

    /// The mouse cursor left the window.
    #[display(fmt = "CursorLeft")]
    CursorLeft,

    /// The mouse cursor was moved
    #[display(fmt = "CursorMoved")]
    CursorMoved {
        /// The new position of the mouse cursor
        position: Pos,
    },

    /// A mouse button was pressed.
    #[display(fmt = "ButtonPressed")]
    ButtonPressed(Button),

    /// A mouse button was released.
    #[display(fmt = "ButtonReleased")]
    ButtonReleased(Button),

    /// The mouse wheel was scrolled.
    #[display(fmt = "WheelScrolled")]
    WheelScrolled {
        /// The scroll movement.
        delta: ScrollDelta,
    },
}

/// A scroll movement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollDelta {
    /// A line-based scroll movement
    Lines {
        /// The number of horizontal lines scrolled
        x: f32,

        /// The number of vertical lines scrolled
        y: f32,
    },
    /// A pixel-based scroll movement
    Pixels {
        /// The number of horizontal pixels scrolled
        x: f32,
        /// The number of vertical pixels scrolled
        y: f32,
    },
}
