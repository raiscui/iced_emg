use crate::PaintCtx;
use emg_native::{Program, Renderer, Widget};
use emg_state::StateAnchor;
use std::ops::Deref;

use crate::GTreeBuilderElement;

/*
 * @Author: Rais
 * @Date: 2022-08-23 11:49:02
 * @LastEditTime: 2022-08-29 16:18:02
 * @LastEditors: Rais
 * @Description:
 */
pub trait GraphProgram: Program {
    type Renderer: Renderer<ImplRenderContext = <Self as Program>::ImplRenderContext>;

    type GTreeBuilder: crate::GTreeBuilderFn<
            <Self as Program>::Message,
            <Self as Program>::ImplRenderContext,
            GraphType = Self::GraphType,
        > + Clone;
    type GraphType: Default;
    type GElementType: Widget<<Self as Program>::Message, <Self as Program>::ImplRenderContext>;
    type RefedGelType: Deref<Target = Self::GElementType>;

    fn tree_build(
        &self,
        // orders: impl Orders<Self::Message> + 'static,
    ) -> GTreeBuilderElement<Self::Message, Self::ImplRenderContext>;

    fn graph_setup(&self, renderer: &Self::Renderer) -> Self::GTreeBuilder;

    // fn view(&self, g: &Self::GraphType) -> Self::RefedGelType;
    fn ctx(&self, g: &Self::GraphType) -> StateAnchor<PaintCtx<Self::ImplRenderContext>>;
}
