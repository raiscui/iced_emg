/*
 * @Author: Rais
 * @Date: 2022-12-22 16:18:38
 * @LastEditTime: 2023-04-13 17:26:44
 * @LastEditors: Rais
 * @Description:
 */

use std::ops::Deref;

use emg_native::renderer::{Affine, BlendMode, BrushRef, Fill, Shape, Stroke};
use vello::{Scene, SceneBuilder as VelloSceneBuilder, SceneFragment as VelloSceneFragment};

#[derive(Default, PartialEq)]
pub struct SceneFrag(pub(crate) VelloSceneFragment, pub(crate) Option<Affine>);

impl Deref for SceneFrag {
    type Target = VelloSceneFragment;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl emg_native::renderer::SceneCtx for SceneFrag {
    type Ctx<'a> = SceneBuilder<'a>;

    fn new(transform: Option<Affine>) -> Self {
        Self(VelloSceneFragment::new(), transform)
    }
    fn gen_builder(&mut self) -> Self::Ctx<'_> {
        SceneBuilder::for_fragment(&mut self.0)
    }

    fn get_transform(&self) -> Option<Affine> {
        self.1
    }
}

pub struct SceneBuilder<'a>(VelloSceneBuilder<'a>);

impl<'a> SceneBuilder<'a> {
    /// Creates a new builder for filling a scene. Any current content in the scene
    /// will be cleared.
    pub fn for_scene(scene: &'a mut Scene) -> Self {
        Self(VelloSceneBuilder::for_scene(scene))
    }

    /// Creates a new builder for filling a scene fragment. Any current content in
    /// the fragment will be cleared.
    pub fn for_fragment(fragment: &'a mut VelloSceneFragment) -> Self {
        Self(VelloSceneBuilder::for_fragment(fragment))
    }
}
// impl<'a> Deref for SceneBuilder<'a> {
//     type Target = VelloSceneBuilder<'a>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
// impl<'a> DerefMut for SceneBuilder<'a> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

impl<'a> emg_native::renderer::SceneBuilder for SceneBuilder<'a> {
    type SceneCtx = SceneFrag;

    #[inline]
    fn push_layer(
        &mut self,
        blend: impl Into<BlendMode>,
        alpha: f32,
        transform: Affine,
        shape: &impl Shape,
    ) {
        self.0.push_layer(blend, alpha, transform, shape)
    }

    #[inline]
    fn pop_layer(&mut self) {
        self.0.pop_layer()
    }

    #[inline]
    fn fill<'b>(
        &mut self,
        style: Fill,
        transform: Affine,
        brush: impl Into<BrushRef<'b>>,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) {
        self.0.fill(style, transform, brush, brush_transform, shape)
    }

    #[inline]
    fn stroke<'b>(
        &mut self,
        style: &Stroke,
        transform: Affine,
        brush: impl Into<BrushRef<'b>>,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) {
        self.0
            .stroke(style, transform, brush, brush_transform, shape)
    }

    #[inline]
    fn append(&mut self, fragment: &Self::SceneCtx, transform: Option<Affine>) {
        self.0.append(fragment, transform)
    }

    #[inline]
    fn draw_image(&mut self, image: &emg_native::renderer::Image, transform: Affine) {
        self.0.draw_image(image, transform)
    }

    #[inline]
    fn draw_glyphs(&mut self, font: &emg_native::renderer::Font) -> vello::DrawGlyphs {
        self.0.draw_glyphs(font)
    }
}
