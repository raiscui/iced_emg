/*
 * @Author: Rais
 * @Date: 2021-08-31 16:05:02
 * @LastEditTime: 2022-08-23 00:30:10
 * @LastEditors: Rais
 * @Description:
 */
// ────────────────────────────────────────────────────────────────────────────────

// pub mod button;
// pub mod checkbox;
// pub use checkbox::Checkbox;
// pub mod layer;
// pub mod text;
// // ────────────────────────────────────────────────────────────────────────────────

// pub use button::Button;
// pub use layer::Layer;
// use seed_styles::GlobalStyleSV;
// pub use text::Text;
// ────────────────────────────────────────────────────────────────────────────────

// ────────────────────────────────────────────────────────────────────────────────

use dyn_clone::DynClone;
use emg_common::dyn_partial_eq::DynPartialEq;

use crate::Bus;

pub trait Widget<Message, RenderContext>: DynClone + DynPartialEq
where
    RenderContext: crate::RenderContext,
{
    // fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size;

    // fn xx(&self, _bus: &Bus<Message>) {}
    fn paint(&self, ctx: &mut crate::PaintCtx<RenderContext>);
}

impl<Message, RenderContext> core::cmp::Eq for dyn Widget<Message, RenderContext> + '_ {}

impl<Message, RenderContext> core::cmp::PartialEq for dyn Widget<Message, RenderContext> + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.box_eq(other.as_any())
    }
}
impl<Message: 'static, RenderContext: 'static> PartialEq<dyn Widget<Message, RenderContext>>
    for Box<dyn Widget<Message, RenderContext>>
{
    fn eq(&self, other: &dyn Widget<Message, RenderContext>) -> bool {
        self.box_eq(other.as_any())
    }
}

dyn_clone::clone_trait_object!(<Message,RenderContext> Widget<Message,RenderContext>);
