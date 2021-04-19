use std::convert::TryInto;

use crate::runtime::{
    css,
    dodrio::{
        self,
        builder::ElementBuilder,
        bumpalo::{self, Bump},
        Attribute, Listener, Node,
    },
    Bus, Css, Element, Length, Widget,
};

// ────────────────────────────────────────────────────────────────────────────────

#[allow(unused_imports)]
use crate::runtime::dodrio::builder::div;
// ────────────────────────────────────────────────────────────────────────────────

/// A container that distributes its contents vertically.
///
/// A [`Layer`] will try to fill the horizontal space of its container.
#[allow(missing_debug_implementations)]
#[derive(Clone, Debug)]
pub struct Layer<'a, Message> {
    id: String,
    width: Length,
    height: Length,
    children: Vec<Element<'a, Message>>,
}

impl<'a, Message> Default for Layer<'a, Message> {
    fn default() -> Self {
        Self::new("")
    }
}

impl<'a, Message> Layer<'a, Message> {
    /// Creates an empty [`Layer`].
    pub fn new<T: Into<String>>(id: T) -> Self {
        Self::with_children(id, Vec::new())
    }

    /// Creates a [`Layer`] with the given elements.
    pub fn with_children<T: Into<String>>(id: T, children: Vec<Element<'a, Message>>) -> Self {
        Layer {
            id: id.into(),
            width: Length::Fill,
            height: Length::Shrink,
            children,
        }
    }

    #[must_use]
    pub fn set_children(mut self, children: Vec<Element<'a, Message>>) -> Self {
        self.children = children;
        self
    }

    /// Sets the vertical spacing _between_ elements.
    ///
    /// Custom margins per element do not exist in Iced. You should use this
    /// method instead! While less flexible, it helps you keep spacing between
    /// elements consistent.

    /// Sets the width of the [`Layer`].
    #[must_use]
    pub const fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Layer`].
    #[must_use]
    pub const fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    pub fn push<E>(mut self, child: E) -> Self
    where
        E: Into<Element<'a, Message>>,
    {
        self.children.push(child.into());
        self
    }
    pub fn ref_push<E>(&mut self, child: E) -> &mut Self
    where
        E: Into<Element<'a, Message>>,
    {
        self.children.push(child.into());
        self
    }
    pub fn try_ref_push<E>(&mut self, child: E) -> &mut Self
    where
        E: TryInto<Element<'a, Message>, Error = ()>,
    {
        //TODO type error,  show error if need;
        if let Ok(e) = child.try_into() {
            self.children.push(e);
        }
        self
    }

    // pub fn update_use<T>(mut self, updater: T) -> Self
    // where
    //     T: crate::RtUpdateFor<Self>,
    // {
    //     updater.refresh_for(&mut self);
    //     self
    // }
}

// impl<'a, Message> crate::UpdateUse for Layer<'a, Message> {
//     fn update_use(mut self, updater: Rc<dyn crate::RtUpdateFor<Self>>) -> Self {
//         updater.refresh_for(&mut self);
//         self
//     }
// }

#[inline]
fn layer<'a, B>(
    // tag_name: &'a str,
    bump: B,
) -> ElementBuilder<
    'a,
    bumpalo::collections::Vec<'a, Listener<'a>>,
    bumpalo::collections::Vec<'a, Attribute<'a>>,
    bumpalo::collections::Vec<'a, Node<'a>>,
>
where
    B: Into<&'a Bump>,
{
    ElementBuilder::new(bump, "layer")
    // ElementBuilder::new(bump, stringify!(layer))
}

// ────────────────────────────────────────────────────────────────────────────────
use crate::NodeBuilder;

impl<'a, Message> NodeBuilder<Message> for Layer<'a, Message>
where
    Message: 'static + Clone,
{
    fn make_element_builder<'b>(
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
        let children: Vec<_> = self
            .children
            .iter()
            .map(|element| element.node(bump, bus, style_sheet))
            .collect();

        // TODO: Complete styling
        layer(
            // bumpalo::format!(in bump,"{}{}",&self.id,"-layer").into_bump_str(),
            bump,
        )
        // .attr(
        //     "class",
        //     bumpalo::format!(in bump, "{} {}", spacing_class, padding_class)
        //         .into_bump_str(),
        // )
        .attr(
            "index",
            bumpalo::collections::String::from_str_in(self.id.as_str(), bump).into_bump_str(),
        )
        .attr(
            "style",
            bumpalo::format!(
                in bump,
                "width: {}; height: {}; display: block; position: absolute",
                css::length(self.width),
                css::length(self.height)
            )
            .into_bump_str(),
        )
        .children(bumpalo::collections::Vec::from_iter_in(
            children.into_iter(),
            bump,
        ))
    }
}
// ────────────────────────────────────────────────────────────────────────────────

impl<'a, Message> Widget<Message> for Layer<'a, Message>
where
    Message: Clone,
{
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        publish: &Bus<Message>,
        style_sheet: &mut Css<'b>,
    ) -> dodrio::Node<'b> {
        let children: Vec<_> = self
            .children
            .iter()
            .map(|element| element.node(bump, publish, style_sheet))
            .collect();

        // TODO: Complete styling
        layer(
            // bumpalo::format!(in bump,"{}{}",&self.id,"-layer").into_bump_str(),
            bump,
        )
        // .attr(
        //     "class",
        //     bumpalo::format!(in bump, "{} {}", spacing_class, padding_class)
        //         .into_bump_str(),
        // )
        .attr(
            "index",
            bumpalo::collections::String::from_str_in(self.id.as_str(), bump).into_bump_str(),
        )
        .attr(
            "style",
            bumpalo::format!(
                in bump,
                "width: {}; height: {}; display: block; position: absolute",
                css::length(self.width),
                css::length(self.height)
            )
            .into_bump_str(),
        )
        .children(children)
        .finish()
    }
}

impl<'a, Message> From<Layer<'a, Message>> for Element<'a, Message>
where
    Message: 'static + Clone,
{
    fn from(layer: Layer<'a, Message>) -> Element<'a, Message> {
        Element::new(layer)
    }
}

// /// NOTE: example for UpdateUse<Who> not Self
// impl<'a, Message> UpdateUse<GElement<'a, Message>> for Layer<'a, Message>
// where
//     Message: 'static + Clone,
// {
//     // type Who = S;
//     default fn update_use(&mut self, updater: &dyn RtUpdateFor<GElement<'a, Message>>) {
//         let nl = self.clone();
//         let mut ge = GElement::GContainer(nl);

//         updater.refresh_for(&mut ge);

//         if let GElement::GContainer(nge) = ge {
//             *self = nge;
//         }
//     }
// }
