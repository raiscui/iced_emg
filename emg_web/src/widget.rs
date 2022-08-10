/*
 * @Author: Rais
 * @Date: 2021-08-31 16:05:02
 * @LastEditTime: 2022-08-09 13:29:58
 * @LastEditors: Rais
 * @Description:
 */
// ────────────────────────────────────────────────────────────────────────────────

pub mod button;
pub mod checkbox;
pub use checkbox::Checkbox;
pub mod layer;
pub mod text;
// ────────────────────────────────────────────────────────────────────────────────

pub use button::Button;
pub use layer::Layer;
use seed_styles::GlobalStyleSV;
pub use text::Text;
// ────────────────────────────────────────────────────────────────────────────────

use crate::Bus;
// ────────────────────────────────────────────────────────────────────────────────
use dodrio::{builder::ElementBuilder, bumpalo, Attribute, Listener};

use dyn_clone::DynClone;
use emg_common::dyn_partial_eq::DynPartialEq;

pub trait Widget<Message>: DynClone + DynPartialEq // where
//     Message: Clone,
{
    fn has_generate_element_builder(&self) -> bool {
        false
    }
    fn generate_element_builder<'b>(
        &self,
        _bump: &'b bumpalo::Bump,
        _bus: &Bus<Message>,
        _style_sheet: &GlobalStyleSV,
    ) -> ElementBuilder<
        'b,
        bumpalo::collections::Vec<'b, Listener<'b>>,
        bumpalo::collections::Vec<'b, Attribute<'b>>,
        bumpalo::collections::Vec<'b, dodrio::Node<'b>>,
    > {
        panic!("need implementation generate_element_builder")
    }

    /// Produces a VDOM node for the [`Widget`].
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
        style_sheet: &GlobalStyleSV,
    ) -> dodrio::Node<'b> {
        self.generate_element_builder(bump, bus, style_sheet)
            .finish()
    }
}

impl<Message> core::cmp::Eq for dyn Widget<Message> + '_ {}

impl<Message> core::cmp::PartialEq for dyn Widget<Message> + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.box_eq(other.as_any())
    }
}
impl<Message: 'static> core::cmp::PartialEq<dyn Widget<Message>> for Box<dyn Widget<Message>> {
    fn eq(&self, other: &dyn Widget<Message>) -> bool {
        self.box_eq(other.as_any())
    }
}

dyn_clone::clone_trait_object!(<Message> Widget<Message>);
