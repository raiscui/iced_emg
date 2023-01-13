/*
 * @Author: Rais
 * @Date: 2022-08-23 11:49:02
 * @LastEditTime: 2023-01-13 11:56:29
 * @LastEditors: Rais
 * @Description:
 */

use emg_common::{Pos, Vector};
use emg_native::{event::EventWithFlagType, renderer::Renderer, Program, Widget};
use emg_state::StateAnchor;
use std::{ops::Deref, rc::Rc};

use crate::GTreeBuilderElement;

pub trait GraphProgram: Program {
    // type Renderer: Renderer<SceneCtx = <Self as Program>::WhoImplSceneCtx>;
    type Renderer: Renderer;

    type GTreeBuilder: crate::GTreeBuilderFn<<Self as Program>::Message, GraphType = Self::GraphType>
        + Clone;
    type GraphType: Default;
    type GElementType: Widget;
    type RefedGelType: Deref<Target = Self::GElementType>;

    fn tree_build(
        &self,
        // orders: impl Orders<Self::Message> + 'static,
    ) -> GTreeBuilderElement<Self::Message>;

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
        StateAnchor<Rc<<Self::Renderer as Renderer>::SceneCtx>>,
    );
}
