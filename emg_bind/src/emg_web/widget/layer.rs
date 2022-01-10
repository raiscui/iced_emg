use std::convert::TryInto;

use emg_core::IdStr;
use seed_styles::GlobalStyleSV;
use tracing::warn;

use crate::emg_runtime::{
    dodrio::{
        self,
        builder::ElementBuilder,
        bumpalo::{self, Bump},
        Attribute, Listener, Node,
    },
    Bus, Element, NodeBuilder, Widget,
};

// ────────────────────────────────────────────────────────────────────────────────

#[allow(unused_imports)]
use crate::emg_runtime::dodrio::builder::div;
// ────────────────────────────────────────────────────────────────────────────────

/// A container that distributes its contents vertically.
///
/// A [`Layer`] will try to fill the horizontal space of its container.
#[allow(missing_debug_implementations)]
#[derive(Clone, Debug)]
pub struct Layer<Message> {
    id: IdStr,
    children: Vec<Element<Message>>,
}

impl<Message> Default for Layer<Message> {
    fn default() -> Self {
        Self::new(IdStr::new_inline(""))
    }
}

impl<Message> Layer<Message> {
    /// Creates an empty [`Layer`].
    pub fn new(id: IdStr) -> Self {
        Self::with_children(id, Vec::new())
    }

    /// Creates a [`Layer`] with the given elements.
    pub fn with_children(id: IdStr, children: Vec<Element<Message>>) -> Self {
        Self { id: id, children }
    }

    #[must_use]
    pub fn set_children(mut self, children: Vec<Element<Message>>) -> Self {
        self.children = children;
        self
    }

    // /// Sets the width of the [`Layer`].
    // #[must_use]
    // pub const fn width(mut self, width: Length) -> Self {
    //     self.width = width;
    //     self
    // }

    // /// Sets the height of the [`Layer`].
    // #[must_use]
    // pub const fn height(mut self, height: Length) -> Self {
    //     self.height = height;
    //     self
    // }

    pub fn push<E>(mut self, child: E) -> Self
    where
        E: Into<Element<Message>>,
    {
        self.children.push(child.into());
        self
    }
    pub fn ref_push<E>(&mut self, child: E) -> &mut Self
    where
        E: Into<Element<Message>>,
    {
        self.children.push(child.into());
        self
    }

    /// `GElement::Refresher_(_)` `GElement::Event_(_)` can't convert to Element
    pub fn try_ref_push<E>(&mut self, child: E) -> Option<&mut Self>
    where
        E: TryInto<Element<Message>, Error = ()>,
    {
        //TODO type error,  show error if need;
        if let Ok(e) = child.try_into() {
            self.children.push(e);
        } else {
            return None;
        }
        Some(self)
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
fn layer<'b, B>(
    // tag_name: &'a str,
    bump: B,
) -> ElementBuilder<
    'b,
    bumpalo::collections::Vec<'b, Listener<'b>>,
    bumpalo::collections::Vec<'b, Attribute<'b>>,
    bumpalo::collections::Vec<'b, Node<'b>>,
>
where
    B: Into<&'b Bump>,
{
    ElementBuilder::new(bump, "layer")
    // ElementBuilder::new(bump, stringify!(layer))
}

// ────────────────────────────────────────────────────────────────────────────────

impl<Message> NodeBuilder<Message> for Layer<Message>
// where
// Message: 'static,
{
    fn generate_element_builder<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
        style_sheet: &GlobalStyleSV,
    ) -> ElementBuilder<
        'b,
        //TODO: replace use Vec or im::Vector, match node() replace node() fn logic.
        bumpalo::collections::Vec<'b, Listener<'b>>,
        bumpalo::collections::Vec<'b, Attribute<'b>>,
        bumpalo::collections::Vec<'b, Node<'b>>,
    > {
        warn!("Layer index:{}", self.id.as_str());

        let children = self
            .children
            .iter()
            .map(|element| element.node(bump, bus, style_sheet));

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
            bumpalo::collections::String::from_str_in("display: block; position: absolute;", bump)
                .into_bump_str(),
        )
        .children(bumpalo::collections::Vec::from_iter_in(children, bump))
    }
}
// ────────────────────────────────────────────────────────────────────────────────

impl<Message> Widget<Message> for Layer<Message>
where
    Message: Clone,
{
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        publish: &Bus<Message>,
        style_sheet: &GlobalStyleSV,
    ) -> dodrio::Node<'b> {
        self.generate_element_builder(bump, publish, style_sheet)
            .finish()
    }
}

impl<Message> From<Layer<Message>> for Element<Message>
where
    Message: 'static + Clone,
{
    fn from(layer: Layer<Message>) -> Self {
        Self::new(layer)
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
