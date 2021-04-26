//! Allow your users to perform actions by pressing a button.
//!
//! A [`Button`] has some local [`State`].

pub use iced_style::button::{Style, StyleSheet};

use crate::runtime::{
    css,
    dodrio::{self, bumpalo},
    Background, Bus, Css, Element, Length, Widget,
};

/// A generic widget that produces a message when pressed.
///
/// ```
/// # use iced_web::{button, Button, Text};
/// #
/// enum Message {
///     ButtonPressed,
/// }
///
/// let mut state = button::State::new();
/// let button = Button::new(&mut state, Text::new("Press me!"))
///     .on_press(Message::ButtonPressed);
/// ```
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct Button<'a, Message> {
    content: Element<'a, Message>,
    on_press: Option<Message>,
    width: Length,
    #[allow(dead_code)]
    height: Length,
    min_width: u32,
    #[allow(dead_code)]
    min_height: u32,
    padding: u16,
    style: Box<dyn StyleSheet>,
}

impl<'a, Message> Button<'a, Message> {
    /// Creates a new [`Button`] with some local [`State`] and the given
    /// content.
    // pub fn new<E>(_state: &'a mut State, content: E) -> Self
    pub fn new<E>(content: E) -> Self
    where
        E: Into<Element<'a, Message>>,
    {
        Button {
            content: content.into(),
            on_press: None,
            width: Length::Shrink,
            height: Length::Shrink,
            min_width: 0,
            min_height: 0,
            padding: 5,
            style: std::boxed::Box::default(),
        }
    }

    /// Sets the width of the [`Button`].
    pub const fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Button`].
    pub const fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the minimum width of the [`Button`].
    pub const fn min_width(mut self, min_width: u32) -> Self {
        self.min_width = min_width;
        self
    }

    /// Sets the minimum height of the [`Button`].
    pub const fn min_height(mut self, min_height: u32) -> Self {
        self.min_height = min_height;
        self
    }

    /// Sets the padding of the [`Button`].
    pub const fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the style of the [`Button`].
    pub fn style(mut self, style: impl Into<Box<dyn StyleSheet>>) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed.
    #[allow(clippy::missing_const_for_fn)]
    pub fn on_press(mut self, msg: Message) -> Self {
        self.on_press = Some(msg);
        self
    }
}

/// The local state of a [`Button`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State;

impl State {
    /// Creates a new [`State`].
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }
}
// ────────────────────────────────────────────────────────────────────────────────
use crate::NodeBuilder;
use iced_web::dodrio::{builder::ElementBuilder, Attribute, Listener, Node};

impl<'a, Message> NodeBuilder<Message> for Button<'a, Message>
where
    Message: 'static + Clone,
{
    fn generate_element_builder<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
        style_sheet: &mut Css<'b>,
    ) -> ElementBuilder<
        'b,
        bumpalo::collections::Vec<'b, Listener<'b>>,
        bumpalo::collections::Vec<'b, Attribute<'b>>,
        bumpalo::collections::Vec<'b, Node<'b>>,
    > {
        use dodrio::builder::button;

        // TODO: State-based styling
        let style = self.style.active();

        let padding_class = style_sheet.insert(bump, css::Rule::Padding(self.padding));

        let background = match style.background {
            None => String::from("none"),
            Some(background) => match background {
                Background::Color(color) => css::color(color),
            },
        };

        let class = {
            use dodrio::bumpalo::collections::String;

            String::from_str_in(&padding_class, bump).into_bump_str()
        };

        let mut node = button(bump)
            .attr("class", class)
            .attr(
                "style",
                bumpalo::format!(
                    in bump,
                    "background: {}; border-radius: {}px; width:{}; \
                    min-width: {}; color: {};",
                    background,
                    style.border_radius,
                    css::length(self.width),
                    css::min_length(self.min_width),
                    css::color(style.text_color)
                )
                .into_bump_str(),
            )
            .children(bumpalo::vec![in bump;self.content.node(bump, bus, style_sheet)]);

        if let Some(on_press) = self.on_press.clone() {
            let event_bus = bus.clone();

            node = node.on("click", move |_root, _vdom, _event| {
                event_bus.publish(on_press.clone());
            });
        }
        node
    }
}

impl<'a, Message> Widget<Message> for Button<'a, Message>
where
    Message: 'static + Clone,
{
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
        style_sheet: &mut Css<'b>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::button;

        // TODO: State-based styling
        let style = self.style.active();

        let padding_class = style_sheet.insert(bump, css::Rule::Padding(self.padding));

        let background = match style.background {
            None => String::from("none"),
            Some(background) => match background {
                Background::Color(color) => css::color(color),
            },
        };

        let class = {
            use dodrio::bumpalo::collections::String;

            String::from_str_in(&padding_class, bump).into_bump_str()
        };

        let mut node = button(bump)
            .attr("class", class)
            .attr(
                "style",
                bumpalo::format!(
                    in bump,
                    "background: {}; border-radius: {}px; width:{}; \
                    min-width: {}; color: {};",
                    background,
                    style.border_radius,
                    css::length(self.width),
                    css::min_length(self.min_width),
                    css::color(style.text_color)
                )
                .into_bump_str(),
            )
            .children(vec![self.content.node(bump, bus, style_sheet)]);

        if let Some(on_press) = self.on_press.clone() {
            let event_bus = bus.clone();

            node = node.on("click", move |_root, _vdom, _event| {
                event_bus.publish(on_press.clone());
            });
        }

        node.finish()
    }
}

impl<'a, Message> From<Button<'a, Message>> for Element<'a, Message>
where
    Message: 'static + Clone,
{
    fn from(button: Button<'a, Message>) -> Element<'a, Message> {
        Element::new(button)
    }
}
