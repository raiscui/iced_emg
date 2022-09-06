/*
 * @Author: Rais
 * @Date: 2021-08-31 16:05:02
 * @LastEditTime: 2022-09-06 12:23:08
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
use emg_common::{dyn_partial_eq::DynPartialEq, IdStr};
use emg_state::StateAnchor;

use crate::Bus;

pub trait Widget<Message, RenderCtx>: DynClone + DynPartialEq
where
    RenderCtx: crate::renderer::RenderContext,
{
    // fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size;

    // fn xx(&self, _bus: &Bus<Message>) {}
    // fn paint(&self, ctx: &mut crate::PaintCtx<RenderContext>);
    // fn paint(&self, ctx: &mut crate::PaintCtx<RenderContext>);
    fn paint_sa(
        &self,
        ctx: &StateAnchor<crate::PaintCtx<RenderCtx>>,
    ) -> StateAnchor<crate::PaintCtx<RenderCtx>>;
}

impl<Message, RenderCtx> core::cmp::Eq for dyn Widget<Message, RenderCtx> + '_ {}

impl<Message, RenderCtx> core::cmp::PartialEq for dyn Widget<Message, RenderCtx> + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.box_eq(other.as_any())
    }
}
impl<Message: 'static, RenderCtx: 'static> PartialEq<dyn Widget<Message, RenderCtx>>
    for Box<dyn Widget<Message, RenderCtx>>
{
    fn eq(&self, other: &dyn Widget<Message, RenderCtx>) -> bool {
        self.box_eq(other.as_any())
    }
}

dyn_clone::clone_trait_object!(<Message,RenderCtx> Widget<Message,RenderCtx>);
