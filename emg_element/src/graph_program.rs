use core::borrow;
use std::{cell::RefCell, ops::Deref, rc::Rc};

use emg_native::{Program, RenderContext, Widget};
use tracing::instrument;

use crate::GTreeBuilderElement;

/*
 * @Author: Rais
 * @Date: 2022-08-23 11:49:02
 * @LastEditTime: 2022-08-23 23:51:02
 * @LastEditors: Rais
 * @Description:
 */
pub trait GraphProgram: Program {
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

    fn graph_setup(&self) -> Self::GTreeBuilder;

    fn view(&self, g: &Self::GraphType) -> Self::RefedGelType;
}
