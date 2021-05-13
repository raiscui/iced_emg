/*
* @Author: Rais
* @Date: 2021-05-07 13:46:16
 * @LastEditTime: 2021-05-07 15:56:50
 * @LastEditors: Rais
* @Description:
*/

use crate::NodeBuilder;
use iced_web::dodrio::bumpalo;
use iced_web::{
    css, Bus, Color, Css, Element, Font, HorizontalAlignment, Length, VerticalAlignment, Widget,
};

/// A paragraph of text.
///
/// # Example
///
/// ```
/// # use iced_web::Text;
///
/// Text::new("I <3 iced!")
///     .size(40);
/// ```
#[derive(Debug, Clone)]
pub struct Text {
    content: String,
    size: Option<u16>,
    color: Option<Color>,
    font: Font,
    width: Length,
    height: Length,
    horizontal_alignment: HorizontalAlignment,
    vertical_alignment: VerticalAlignment,
}

impl Text {
    /// Create a new fragment of [`Text`] with the given contents.
    pub fn new<T: Into<String>>(label: T) -> Self {
        Self {
            content: label.into(),
            size: None,
            color: None,
            font: Font::Default,
            width: Length::Shrink,
            height: Length::Shrink,
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
        }
    }

    /// update content string of the [`Text`].
    pub fn content<T: Into<String>>(&mut self, label: T) -> &mut Self {
        self.content = label.into();
        self
    }

    /// Sets the size of the [`Text`].
    #[must_use]
    pub const fn size(mut self, size: u16) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets the [`Color`] of the [`Text`].
    pub fn color<C: Into<Color>>(mut self, color: C) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Sets the [`Font`] of the [`Text`].
    #[must_use]
    pub const fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    /// Sets the width of the [`Text`] boundaries.
    #[must_use]
    pub const fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Text`] boundaries.
    #[must_use]
    pub const fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the [`HorizontalAlignment`] of the [`Text`].
    #[must_use]
    pub const fn horizontal_alignment(mut self, alignment: HorizontalAlignment) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    /// Sets the [`VerticalAlignment`] of the [`Text`].
    #[must_use]
    pub const fn vertical_alignment(mut self, alignment: VerticalAlignment) -> Self {
        self.vertical_alignment = alignment;
        self
    }
}

impl<'a, Message> Widget<Message> for Text {
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        publish: &Bus<Message>,
        style_sheet: &mut Css<'b>,
    ) -> iced_web::dodrio::Node<'b> {
        self.generate_element_builder(bump, publish, style_sheet)
            .finish()
    }
}

impl<'a, Message> From<Text> for Element<'a, Message> {
    fn from(text: Text) -> Element<'a, Message> {
        Element::new(text)
    }
}

impl<Message> NodeBuilder<Message> for Text
// where
//     Message: 'static,
{
    fn generate_element_builder<'b>(
        &self,
        bump: &'b iced_web::dodrio::bumpalo::Bump,
        _bus: &iced_web::Bus<Message>,
        _style_sheet: &mut iced_web::Css<'b>,
    ) -> iced_web::dodrio::builder::ElementBuilder<
        'b,
        iced_web::dodrio::bumpalo::collections::Vec<'b, iced_web::dodrio::Listener<'b>>,
        iced_web::dodrio::bumpalo::collections::Vec<'b, iced_web::dodrio::Attribute<'b>>,
        iced_web::dodrio::bumpalo::collections::Vec<'b, iced_web::dodrio::Node<'b>>,
    > {
        use iced_web::dodrio::builder::{p, text};

        let content = {
            use iced_web::dodrio::bumpalo::collections::String;

            String::from_str_in(&self.content, bump)
        };

        let color = self.color.map_or(String::from("inherit"), css::color);

        let width = css::length(self.width);
        let height = css::length(self.height);

        let text_align = match self.horizontal_alignment {
            HorizontalAlignment::Left => "left",
            HorizontalAlignment::Center => "center",
            HorizontalAlignment::Right => "right",
        };

        let style = bumpalo::format!(
            in bump,
            "display: block; position: absolute; width: {}; height: {}; font-size: {}px; color: {}; \
            text-align: {}; font-family: {};",
            width,
            height,
            self.size.unwrap_or(20),
            color,
            text_align,
            match self.font {
                Font::Default => "inherit",
                Font::External { name, .. } => name,
            }
        );

        // TODO: Complete styling
        p(bump)
            .attr("style", style.into_bump_str())
            .children(bumpalo::vec![in bump;text(content.into_bump_str())])
    }
}
