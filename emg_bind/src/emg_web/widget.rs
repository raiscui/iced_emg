/*
 * @Author: Rais
 * @Date: 2021-08-31 16:05:02
 * @LastEditTime: 2021-09-02 17:37:17
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

pub trait Widget<Message>: DynClone // where
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
dyn_clone::clone_trait_object!(<Message> Widget<Message>);
