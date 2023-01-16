//! Allow your users to perform actions by pressing a button.
//!
//! A [`Button`] has some local [`State`].

use emg_common::LogicLength;
use seed_styles::GlobalStyleSV;
// use crate::iced_runtime::{css, Background, Length};
use crate::widget::Widget;
use crate::{Bus, GElement};

/// A generic widget that produces a message when pressed.
///
/// ```
/// # use emg_bind::{button, Button, Text};
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
#[derive(Clone, PartialEq)]
pub struct Button<Message> {
    // id: String,
    content: Box<GElement<Message>>,
    on_press: Option<Message>,
    width: LogicLength,
    #[allow(dead_code)]
    height: LogicLength,
    min_width: u32,
    #[allow(dead_code)]
    min_height: u32,
    padding: u16,
    // style: Box<dyn StyleSheet>,
}
// impl< Message> ShapingWhoNoWarper for Button< Message> {}
// impl< Message> Shaping<Button< Message>> for Gid {
//     fn shaping(&self, el: &mut Button< Message>) {
//         el.id = self.id();
//     }
// }

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

/// The local state of a [`Button`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State;

impl State {
    /// Creates a new [`State`].
    #[allow(dead_code)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
// ────────────────────────────────────────────────────────────────────────────────
use dodrio::{builder::ElementBuilder, bumpalo, Attribute, Listener, Node};

impl<Message> Widget<Message> for Button<Message>
where
    Message: 'static + Clone + std::cmp::PartialEq,
{
    fn has_generate_element_builder(&self) -> bool {
        true
    }
    fn generate_element_builder<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
        style_sheet: &GlobalStyleSV,
    ) -> ElementBuilder<
        'b,
        bumpalo::collections::Vec<'b, Listener<'b>>,
        bumpalo::collections::Vec<'b, Attribute<'b>>,
        bumpalo::collections::Vec<'b, Node<'b>>,
    > {
        use dodrio::builder::button;

        // TODO: State-based styling
        // let style = self.style.active();

        // let padding_class = style_sheet.insert(bump, css::Rule::Padding(self.padding));

        // let background = match style.background {
        //     None => String::from("none"),
        //     Some(background) => match background {
        //         Background::Color(color) => css::color(color),
        //     },
        // };

        // let class = {
        //     use dodrio::bumpalo::collections::String;

        //     String::from_str_in(&padding_class, bump).into_bump_str()
        // };

        let mut node = button(bump)
            //TODO button style
            // .attr("class", class)
            // .attr(
            //     "style",
            //     bumpalo::format!(
            //         in bump,
            //         "background: {}; border-radius: {}px; width:{}; \
            //         min-width: {}; color: {};",
            //         background,
            //         style.border_radius,
            //         css::length(self.width),
            //         css::min_length(self.min_width),
            //         css::color(style.text_color)
            //     )
            //     .into_bump_str(),
            // )
            .attr(
                "style",
                bumpalo::collections::String::from_str_in(
                    "display: block; position: absolute;",
                    bump,
                )
                .into_bump_str(),
            )
            .children(bumpalo::vec![in bump;self.content.as_dyn_node_widget() .node(bump, bus, style_sheet)]);

        if let Some(on_press) = self.on_press.clone() {
            let event_bus = bus.clone();

            node = node.on("click", move |_root, _vdom, _event| {
                event_bus.publish(on_press.clone());
            });
        }
        node
    }
}
