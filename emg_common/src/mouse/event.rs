use crate::Pos;
// use bitflags_export_const::bitflagsExportConst;
use bitflags::bitflags;

// use crate::Point;
use super::Button;

// Event bigflags

bitflags! {
    /// EventFlag
/// 由于 用户想要触发的事件有多重意图, 比如 click 表示 不管是按下哪个鼠标键都会触发,
/// 所以需要一个标志位来表示多重语义的事件
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct EventFlag: u32 {
        const GENERAL_CLICK =   1<<0;
        const CURSOR =          1<<1;
        const LEFT =            1<<2;
        const RIGHT =           1<<3;
        const PRESSED =         1<<4 | Self::GENERAL_CLICK.bits();
        const RELEASED =        1<<5 | Self::GENERAL_CLICK.bits();
        const CURSOR_MOVED =    1<<6 | self::CURSOR.bits();
        const CURSOR_ENTERED =  1<<7 | self::CURSOR.bits();
        const CURSOR_LEFT =     1<<8 | self::CURSOR.bits();
        const MIDDLE =          1<<9;
        const OTHER_BUTTOM =    1<<10;
        const WHEEL_SCROLLED =  1<<11;
// ────────────────────────────────────────────────────────────────────────────────

        const LEFT_CLICK = Self::GENERAL_CLICK.bits() | Self::LEFT.bits();
        const LEFT_PRESSED= Self::PRESSED.bits() | Self::LEFT.bits();
        const LEFT_RELEASED= Self::RELEASED.bits() | Self::LEFT.bits();
// ────────────────────────────────────────────────────────────────────────────────

        const RIGHT_CLICK = Self::GENERAL_CLICK.bits() | Self::RIGHT.bits();
        const RIGHT_PRESSED = Self::PRESSED.bits() | Self::RIGHT.bits();
        const RIGHT_RELEASED = Self::RELEASED.bits() | Self::RIGHT.bits();
// ────────────────────────────────────────────────────────────────────────────────

        const MIDDLE_CLICK = Self::GENERAL_CLICK.bits() | Self::MIDDLE.bits();
        const MIDDLE_PRESSED = Self::PRESSED.bits() | Self::MIDDLE.bits();
        const MIDDLE_RELEASED = Self::RELEASED.bits() | Self::MIDDLE.bits();
        // ─────────────────────────────────────────────────────────────────

        const OTHER_CLICK = Self::GENERAL_CLICK.bits() | Self::OTHER_BUTTOM.bits();
        const OTHER_PRESSED = Self::PRESSED.bits() | Self::OTHER_BUTTOM.bits();
        const OTHER_RELEASED = Self::RELEASED.bits() | Self::OTHER_BUTTOM.bits();
        // alias ─────────────────────────────────────────────────────────────

        const CLICK = Self::LEFT_RELEASED.bits();
    }
}

/// A mouse event.
///
/// _**Note:** This type is largely incomplete! If you need to track
/// additional events, feel free to [open an issue] and share your use case!_
///
/// [open an issue]: https://github.com/iced-rs/iced/issues
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    /// The mouse cursor entered the window.
    CursorEntered,

    /// The mouse cursor left the window.
    CursorLeft,

    /// The mouse cursor was moved
    CursorMoved {
        /// The new position of the mouse cursor
        position: Pos,
    },

    /// A mouse button was pressed.
    ButtonPressed(Button),

    /// A mouse button was released.
    ButtonReleased(Button),

    /// The mouse wheel was scrolled.
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn xx() {
        let x = CLICK;
        println!("{:?}", x);
    }
}
