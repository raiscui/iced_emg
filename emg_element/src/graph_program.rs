/*
 * @Author: Rais
 * @Date: 2022-08-23 11:49:02
 * @LastEditTime: 2022-09-19 10:16:11
 * @LastEditors: Rais
 * @Description:
 */

use crate::{node_builder::EventNode, PaintCtx};
use emg_common::{IdStr, Pos, Vector};
use emg_native::{event::EventWithFlagType, renderer::Renderer, Event, Program, Widget};
use emg_state::{Dict, StateAnchor};
use std::ops::Deref;

use crate::GTreeBuilderElement;

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
    fn root_id(&self) -> &str;
    fn ctx(
        &self,
        g: &Self::GraphType,
        events: &StateAnchor<Vector<EventWithFlagType>>,
        cursor_position: &StateAnchor<Option<Pos>>,
    ) -> (
        crate::EventMatchsSa<Self::Message>,
        StateAnchor<PaintCtx<Self::ImplRenderContext>>,
    );
}
