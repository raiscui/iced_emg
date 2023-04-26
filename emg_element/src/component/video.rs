/*
 * @Author: Rais
 * @Date: 2023-04-13 13:08:30
 * @LastEditTime: 2023-04-26 11:02:58
 * @LastEditors: Rais
 * @Description:
 */

use std::{clone::Clone, cmp::PartialEq, rc::Rc};

use emg_common::{
    better_any::{Tid, TidAble},
    IdStr,
};
use emg_layout::EmgEdgeItem;
use emg_shaping::Shaping;
use emg_state::{topo, StateAnchor, StateMultiAnchor};
use tracing::Span;

use crate::{g_tree_builder::GTreeInit, platform::renderer::Image, GElement, InitdTree};
// ─────────────────────────────────────────────────────────────────────────────
use crate::platform::features::VideoPlayer;
// ────────────────────────────────────────────────────────────────────────────────
mod control;

pub use control::VideoController;

#[derive(Tid)]
pub struct Video {
    id: IdStr,
    player: Rc<VideoPlayer>,
    //TODO vec?
    // children: LayerChildren<Message>,
}

impl Video {
    pub fn with_setup(mut self, some: &dyn Shaping<Self>) -> Self {
        let _ = some.shaping(&mut self);
        self
    }

    pub fn player(&self) -> &Rc<VideoPlayer> {
        &self.player
    }
}

impl Eq for Video {}

impl Clone for Video {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            player: self.player.clone(),
        }
    }
}

impl std::fmt::Debug for Video {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Video")
            .field("id", &self.id)
            .field("player", &self.player)
            .finish()
    }
}

impl PartialEq for Video {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id &&
        // self.player == other.player
        Rc::ptr_eq(&self.player, &other.player)
    }
}

impl Video {
    #[topo::nested]
    pub fn new(id: impl Into<IdStr>, uri: &str, live: bool) -> Self
// where
        // Self: Sized,
    {
        let player = VideoPlayer::new(uri, live).expect("video_player new fn");
        Self {
            id: id.into(),
            player: Rc::new(player),
        }
    }

    pub fn frame_image_sa(&self) -> &StateAnchor<Image> {
        self.player.frame_image_sa()
    }
}

impl<Message> GTreeInit<Message> for Video
where
    Message: Clone + PartialEq + for<'a> emg_common::any::MessageTid<'a>,
{
    fn tree_init(
        self,
        _id: &IdStr,
        _es: &[Rc<dyn Shaping<EmgEdgeItem>>],
        _children: &[crate::GTreeBuilderElement<Message>],
    ) -> InitdTree<Message> {
        GElement::Video_(self).into()
    }
}

#[cfg(all(feature = "gpu"))]
use crate::platform::renderer::*;
#[cfg(all(feature = "gpu"))]
impl crate::Widget for Video {
    type SceneCtxType = crate::renderer::SceneFrag;
    fn paint_sa(
        &self,
        ctx: &StateAnchor<crate::platform::PaintCtx>,
    ) -> StateAnchor<Rc<Self::SceneCtxType>> {
        // let id = self.id.clone();
        // let span = illicit::expect::<Span>();

        // let player = self.player.clone();
        let frame_sa = self.frame_image_sa();
        let pause = self.player.paused().watch();

        (ctx, frame_sa, &pause).map(move |incoming_ctx, image, &paused| {
            // ─────────────────────────────────────────────────────

            let mut sc = Self::SceneCtxType::new(incoming_ctx.get_translation());
            let mut builder = sc.gen_builder();

            builder.draw_image(image, Affine::IDENTITY);

            Rc::new(sc)
        })
    }
}
