/*
 * @Author: Rais
 * @Date: 2022-12-22 16:18:38
 * @LastEditTime: 2023-01-09 11:32:02
 * @LastEditors: Rais
 * @Description:
 */

use std::ops::{Deref, DerefMut};

use emg_native::renderer::{Affine, BlendMode, BrushRef, Fill, Shape, Stroke};
use vello::{Scene, SceneBuilder as VelloSceneBuilder, SceneFragment as VelloSceneFragment};

// use ouroboros::self_referencing;

// #[self_referencing]
// pub struct SceneCtx {
//     scene: SceneFragment,
//     #[borrows(mut scene)]
//     #[covariant]
//     scene_builder: SceneBuilder<'this>,
// }

// impl Default for SceneCtx {
//     fn default() -> Self {
//         Self::build(Default::default())
//     }
// }

// impl SceneCtx {
//     pub fn build(scene: SceneFragment) -> Self {
//         SceneCtxBuilder {
//             scene,
//             scene_builder_builder: |scene| SceneBuilder::for_fragment(scene),
//         }
//         .build()
//     }
//     pub fn finish(mut self) -> SceneFragment {
//         self.with_scene_builder_mut(|sb| {
//             sb.finish();
//         });
//         self.into_heads().scene
//     }
// }

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

    fn push_layer(
        &mut self,
        blend: impl Into<BlendMode>,
        alpha: f32,
        transform: Affine,
        shape: &impl Shape,
    ) {
        self.0.push_layer(blend, alpha, transform, shape)
    }

    fn pop_layer(&mut self) {
        self.0.pop_layer()
    }

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

    fn append(&mut self, fragment: &Self::SceneCtx, transform: Option<Affine>) {
        self.0.append(fragment, transform)
    }

    fn finish(self) {
        self.0.finish()
    }
}
