use std::{clone::Clone, cmp::PartialEq, convert::TryInto};

use dyn_partial_eq::DynPartialEq;
use emg_core::IdStr;
use seed_styles::GlobalStyleSV;
use tracing::debug;

use crate::{
    emg_runtime::{
        dodrio::{
            builder::ElementBuilder,
            bumpalo::{self, Bump},
            Attribute, Listener, Node,
        },
        Bus, Element, Widget,
    },
    GElement,
};

// ────────────────────────────────────────────────────────────────────────────────

#[allow(unused_imports)]
use crate::emg_runtime::dodrio::builder::div;
// ────────────────────────────────────────────────────────────────────────────────

type LayerChildren<Message> = Vec<GElement<Message>>;

/// A container that distributes its contents vertically.
///
/// A [`Layer`] will try to fill the horizontal space of its container.
#[allow(missing_debug_implementations)]
#[derive(Clone, DynPartialEq, Eq, Debug)]
#[eq_opt(where_add = "Message: PartialEq+'static,")]
pub struct Layer<Message> {
    id: IdStr,
    //TODO vec?
    children: LayerChildren<Message>,
}
impl<Message> PartialEq for Layer<Message>
where
    Message: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.children == other.children
    }
}

// impl<Message> DynPartialEq for Layer<Message> {
//     fn as_any(&self) -> &dyn core::any::Any {
//         self
//     }
//     fn box_eq(&self, other: &dyn core::any::Any) -> bool {
//         other.downcast_ref::<Self>().map_or(false, |a| self == a)
//     }
// }

impl<Message> Default for Layer<Message> {
    fn default() -> Self {
        Self::new(IdStr::new_inline(""))
    }
}

impl<Message> Layer<Message> {
    /// Creates an empty [`Layer`].
    #[must_use]
    pub fn new(id: IdStr) -> Self {
        Self::with_children(id, LayerChildren::<Message>::new())
    }

    /// Creates a [`Layer`] with the given elements.
    #[must_use]
    pub fn with_children(id: IdStr, children: LayerChildren<Message>) -> Self {
        Self { id, children }
    }

    #[must_use]
    pub fn set_children(mut self, children: LayerChildren<Message>) -> Self {
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

    pub fn push(&mut self, child: GElement<Message>) {
        self.children.push(child);
    }
    // pub fn ref_push<E>(&mut self, child: E) -> &mut Self
    // where
    //     E: Into<Element<Message>>,
    // {
    //     self.children.push(child.into());
    //     self
    // }

    // /// `GElement::Refresher_(_)` `GElement::Event_(_)` can't convert to Element
    // pub fn try_ref_push<E>(&mut self, child: E)
    // where
    //     E: TryInto<Element<Message>>,
    // {
    //     //TODO type error,  show error if need;
    //     if let Ok(e) = child.try_into() {
    //         self.children.push(e);
    //     }
    // }

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

impl<Message> Widget<Message> for Layer<Message>
where
    Message: PartialEq + 'static + std::clone::Clone,
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
        //TODO: replace use Vec or im::Vector, match node() replace node() fn logic.
        bumpalo::collections::Vec<'b, Listener<'b>>,
        bumpalo::collections::Vec<'b, Attribute<'b>>,
        bumpalo::collections::Vec<'b, Node<'b>>,
    > {
        debug!("Layer index:{}", self.id.as_str());

        let children = self
            .children
            .iter()
            .map(|element| element.as_dyn_node_widget().node(bump, bus, style_sheet));

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
        //TODO remove this check tests
        // .attr(
        //     "index",
        //     bumpalo::collections::String::from_str_in(self.id.as_str(), bump).into_bump_str(),
        // )
        .attr(
            "style",
            bumpalo::collections::String::from_str_in("display: block; position: absolute;", bump)
                .into_bump_str(),
        )
        .children(bumpalo::collections::Vec::from_iter_in(children, bump))
    }
}
// ────────────────────────────────────────────────────────────────────────────────

// impl<Message> From<Layer<Message>> for Element<Message>
// where
//     Message: Clone + 'static + PartialEq,
// {
//     fn from(layer: Layer<Message>) -> Self {
//         Self::new(layer)
//     }
// }

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
