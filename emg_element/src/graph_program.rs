/*
 * @Author: Rais
 * @Date: 2022-08-23 11:49:02
 * @LastEditTime: 2023-02-01 00:06:25
 * @LastEditors: Rais
 * @Description:
 */

use emg_common::{Pos, Vector};
use emg_native::{event::EventWithFlagType, renderer::Renderer, Program};
use emg_state::StateAnchor;
use std::rc::Rc;

pub trait GraphProgram: Program {
    // type Renderer: Renderer<SceneCtx = <Self as Program>::WhoImplSceneCtx>;
    type Renderer: Renderer;

    type GTreeWithBuilder: crate::GTreeBuilderFn<
            Self::Message,
            GraphType = Self::GraphType,
            GraphEditor = Self::GraphEditor,
        > + Clone;
    // type GElementType: Widget;
    // type RefedGelType: Deref<Target = Self::GElementType>;

    fn graph_setup(
        &self,
        renderer: &Self::Renderer,
        orders: Self::Orders,
    ) -> Self::GTreeWithBuilder;

    // fn view(&self, g: &Self::GraphType) -> Self::RefedGelType;
    fn root_id(&self) -> &str;
    fn build_ctx(
        &self,
        g: &Self::GraphType,
        paint: &StateAnchor<crate::PaintCtx>,
        events: &StateAnchor<Vector<EventWithFlagType>>,
        cursor_position: &StateAnchor<Option<Pos>>,
    ) -> (
        crate::EventMatchsSa<Self::Message>,
        StateAnchor<Rc<<Self::Renderer as Renderer>::SceneCtx>>,
    );
}
