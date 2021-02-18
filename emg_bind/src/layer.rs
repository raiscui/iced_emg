use std::rc::Rc;

use overloadf::*;

use crate::{
    runtime::{
        css,
        dodrio::{
            self,
            builder::ElementBuilder,
            bumpalo::{self, Bump},
            Attribute, Listener, Node,
        },
        Bus, Css, Element, Length, Widget,
    },
    GElement,
    GElement::*,
    RtUpdateFor,
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
    width: Length,
    height: Length,
    children: Vec<Element<'a, Message>>,
}
impl<'a, Message> Layer<'a, Message> {
    /// Creates an empty [`Layer`].
    pub fn new() -> Self {
        Self::with_children(Vec::new())
    }

    /// Creates a [`Layer`] with the given elements.
    pub fn with_children(children: Vec<Element<'a, Message>>) -> Self {
        Layer {
            width: Length::Fill,
            height: Length::Shrink,
            children,
        }
    }

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
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Layer`].
    pub fn height(mut self, height: Length) -> Self {
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

    // pub fn update_use<T>(mut self, updater: T) -> Self
    // where
    //     T: crate::RtUpdateFor<Self>,
    // {
    //     updater.update_for(&mut self);
    //     self
    // }
}

// impl<'a, Message> crate::UpdateUse for Layer<'a, Message> {
//     fn update_use(mut self, updater: Rc<dyn crate::RtUpdateFor<Self>>) -> Self {
//         updater.update_for(&mut self);
//         self
//     }
// }

#[inline]
fn layer<'a, B>(
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
        use crate::runtime::dodrio::builder::*;

        let children: Vec<_> = self
            .children
            .iter()
            .map(|element| element.node(bump, publish, style_sheet))
            .collect();

        // TODO: Complete styling
        layer(bump)
            // .attr(
            //     "class",
            //     bumpalo::format!(in bump, "{} {}", spacing_class, padding_class)
            //         .into_bump_str(),
            // )
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

impl<'a, Message> RtUpdateFor<GElement<'a, Message>> for Layer<'a, Message>
where
    Message: 'static + Clone,
{
    fn update_for(&self, el: &mut GElement<'a, Message>) {
        match el {
            GContainer(l) => {
                log::debug!("layer update use i32");
                l.ref_push(self.clone());
            }
            GSurface(_el) => {
                log::debug!("element update layer");
            }
            GText(text) => {
                log::info!("==========Text update use i32");
                // text.content(format!("i32:{}", self));
            }
            GUpdater(_) => {
                log::debug!("Updater update use i32");
            }
        }
    }
}
