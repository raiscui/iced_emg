/*
 * @Author: Rais
 * @Date: 2021-04-29 10:03:54
 * @LastEditTime: 2021-05-13 18:53:40
 * @LastEditors: Rais
 * @Description:
 */
//! Handle events of a user interface.
use emg_animation::Tick;
use emg_orders::Orders;

// use crate::keyboard;
// use crate::mouse;
// use crate::touch;
use crate::window;

/// A user interface event.
///
/// _**Note:** This type is largely incomplete! If you need to track
/// additional events, feel free to [open an issue] and share your use case!_
///
/// [open an issue]: https://github.com/hecrj/iced/issues
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Event {
    // /// A keyboard event
    // Keyboard(keyboard::Event),

    // /// A mouse event
    // Mouse(mouse::Event),
    /// A window event
    Window(window::Event),
    // /// A touch event
    // Touch(touch::Event),
    OnAnimationFrame(emg_animation::Msg),
}

/// The status of an [`Event`] after being processed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The [`Event`] was **NOT** handled by any widget.
    Ignored,

    /// The [`Event`] was handled and processed by a widget.
    Captured,
}

impl Status {
    /// Merges two [`Status`] into one.
    ///
    /// `Captured` takes precedence over `Ignored`:
    ///
    /// ```
    /// use iced_native::event::Status;
    ///
    /// assert_eq!(Status::Ignored.merge(Status::Ignored), Status::Ignored);
    /// assert_eq!(Status::Ignored.merge(Status::Captured), Status::Captured);
    /// assert_eq!(Status::Captured.merge(Status::Ignored), Status::Captured);
    /// assert_eq!(Status::Captured.merge(Status::Captured), Status::Captured);
    /// ```
    #[must_use]
    pub const fn merge(self, b: Self) -> Self {
        match self {
            Status::Ignored => b,
            Status::Captured => Self::Captured,
        }
    }
}
