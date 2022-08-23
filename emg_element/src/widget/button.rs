use emg_common::LogicLength;

use crate::GElement;

#[allow(missing_debug_implementations)]
#[derive(Clone, PartialEq)]
pub struct Button<Message> {
    // id: String,
    content: Box<GElement<Message>>,
    on_press: Option<Message>,
    width: LogicLength,
    height: LogicLength,
    min_width: u32,
    min_height: u32,
    padding: u16,
    // style: Box<dyn StyleSheet>,
}

impl<Message> Button<Message> {
    /// Creates a new [`Button`] with some local [`State`] and the given
    /// content.
    // pub fn new<E>(_state: &'a mut State, content: E) -> Self
    pub fn new<E>(content: E) -> Self
    where
        E: Into<GElement<Message>>,
    {
        Self {
            // id: "".to_string(),
            content: Box::new(content.into()),
            on_press: None,
            width: LogicLength::default(),
            height: LogicLength::default(),
            min_width: 0,
            min_height: 0,
            padding: 5,
            // style: std::boxed::Box::default(),
        }
    }

    /// Sets the width of the [`Button`].
    #[must_use]
    pub fn width(mut self, width: LogicLength) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Button`].
    #[must_use]
    pub fn height(mut self, height: LogicLength) -> Self {
        self.height = height;
        self
    }

    /// Sets the minimum width of the [`Button`].
    #[must_use]
    pub const fn min_width(mut self, min_width: u32) -> Self {
        self.min_width = min_width;
        self
    }

    /// Sets the minimum height of the [`Button`].
    #[must_use]
    pub const fn min_height(mut self, min_height: u32) -> Self {
        self.min_height = min_height;
        self
    }

    /// Sets the padding of the [`Button`].
    #[must_use]
    pub const fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    // /// Sets the style of the [`Button`].
    // #[must_use]
    // pub fn style(mut self, style: impl Into<Box<dyn StyleSheet>>) -> Self {
    //     self.style = style.into();
    //     self
    // }

    /// Sets the message that will be produced when the [`Button`] is pressed.
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn on_press(mut self, msg: Message) -> Self {
        self.on_press = Some(msg);
        self
    }
}
