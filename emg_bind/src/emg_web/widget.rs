/*
 * @Author: Rais
 * @Date: 2021-08-31 16:05:02
 * @LastEditTime: 2022-06-02 09:12:50
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
use dodrio::bumpalo;

use dyn_clone::DynClone;
use dyn_partial_eq::*;

pub trait Widget<Message>: DynClone + DynPartialEq // where
//     Message: Clone,
{
    /// Produces a VDOM node for the [`Widget`].
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        _bus: &Bus<Message>,
        style_sheet: &GlobalStyleSV,
    ) -> dodrio::Node<'b>;
}

impl<Message> core::cmp::Eq for dyn Widget<Message> + '_ {}

impl<Message> core::cmp::PartialEq for dyn Widget<Message> + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.box_eq(other.as_any())
    }
}
impl<Message> core::cmp::PartialEq<dyn Widget<Message> + '_> for Box<dyn Widget<Message> + '_> {
    fn eq(&self, other: &dyn Widget<Message>) -> bool {
        self.box_eq(other.as_any())
    }
}

dyn_clone::clone_trait_object!(<Message> Widget<Message>);
