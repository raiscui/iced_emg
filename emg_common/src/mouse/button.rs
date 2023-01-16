/// The button of a mouse.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Button {
    /// The left mouse button.
    Left,

    /// The right mouse button.
    Right,

    /// The middle (wheel) button.
    Middle,

    /// Some other button.
    Other(u8),
}

impl Button {
    /// Returns `true` if the button is [`Left`].
    ///
    /// [`Left`]: Button::Left
    #[must_use]
    pub fn is_left(&self) -> bool {
        matches!(self, Self::Left)
    }

    /// Returns `true` if the button is [`Right`].
    ///
    /// [`Right`]: Button::Right
    #[must_use]
    pub fn is_right(&self) -> bool {
        matches!(self, Self::Right)
    }
}
