/*
 * @Author: Rais
 * @Date: 2021-08-31 16:05:02
 * @LastEditTime: 2023-02-01 00:09:12
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

use std::rc::Rc;

use dyn_clone::DynClone;
use emg_common::dyn_partial_eq::DynPartialEq;
use emg_state::StateAnchor;

pub trait Widget: DynClone + DynPartialEq {
    type SceneCtxType;
    // fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size;

    // fn xx(&self, _bus: &Bus<Message>) {}
    // fn paint(&self, ctx: &mut crate::PaintCtx<RenderContext>);
    // fn paint(&self, ctx: &mut crate::PaintCtx<RenderContext>);
    fn paint_sa(
        &self,
        painter: &StateAnchor<crate::PaintCtx>,
    ) -> StateAnchor<Rc<Self::SceneCtxType>>;
}

impl<SceneCtx> core::cmp::Eq for dyn Widget<SceneCtxType = SceneCtx> + '_ {}

impl<SceneCtx> core::cmp::PartialEq for dyn Widget<SceneCtxType = SceneCtx> + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.box_eq(other.as_any())
    }
}
impl<SceneCtx: 'static> PartialEq<dyn Widget<SceneCtxType = SceneCtx>>
    for Box<dyn Widget<SceneCtxType = SceneCtx>>
{
    fn eq(&self, other: &dyn Widget<SceneCtxType = SceneCtx>) -> bool {
        self.box_eq(other.as_any())
    }
}

dyn_clone::clone_trait_object!(<SceneCtx> Widget<SceneCtxType=SceneCtx>);
