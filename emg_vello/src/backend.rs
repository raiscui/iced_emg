/*
 * @Author: Rais
 * @Date: 2022-08-14 15:29:14
 * @LastEditTime: 2023-01-05 16:22:57
 * @LastEditors: Rais
 * @Description:
 */
use vello::{util::RenderContext as VelloRenderContext, Scene, SceneBuilder, SceneFragment};
use vello::{util::RenderSurface as VelloRenderSurface, Renderer as VelloRenderer};

use crate::{scene_ctx::SceneFrag, Settings, NUM_FRAMES};

use emg_graphics_backend::Error;

use tracing::{debug, info};

// #[cfg(any(feature = "image_rs", feature = "svg"))]
// use crate::image;

/// A [`wgpu`] graphics backend for [`elemg`].
pub struct Backend {
    renderer: VelloRenderer,
    current_frame: usize,
    pub scene: Scene,
}

impl Backend {
    /// Creates a new [`Backend`].
    pub fn new(render_cx: &VelloRenderContext) -> Result<Self, Error> {
        let renderer = VelloRenderer::new(&render_cx.device)
            .map_err(|e| Error::BackendError(e.to_string()))?;

        let scene = Scene::new();

        Ok(Self {
            renderer,
            current_frame: 0,
            scene,
        })
    }

    /// Draws the provided primitives in the given `TextureView`.
    pub fn present(&mut self, render_cx: &VelloRenderContext, surface: &VelloRenderSurface) {
        let surface_texture = surface
            .surface
            .get_current_texture()
            .expect("failed to get surface texture");

        self.renderer
            .render_to_surface(
                &render_cx.device,
                &render_cx.queue,
                &self.scene,
                &surface_texture,
                surface.config.width,
                surface.config.height,
            )
            .expect("failed to render to surface");

        surface_texture.present();

        render_cx.device.poll(wgpu::Maintain::Wait);
    }
}

impl emg_graphics_backend::Backend for Backend {
    type SceneCtx = crate::SceneFrag;

    fn new_scene_ctx() -> crate::SceneFrag {
        crate::SceneFrag::default()
    }

    fn on_loop_destroyed(&mut self) {}

    // fn trim_measurements(&mut self) {
    //     self.text_pipeline.trim_measurement_cache()
    // }
}

// impl backend::Text for Backend {
//     const ICON_FONT: Font = font::ICONS;
//     const CHECKMARK_ICON: char = font::CHECKMARK_ICON;
//     const ARROW_DOWN_ICON: char = font::ARROW_DOWN_ICON;

//     fn default_size(&self) -> u16 {
//         self.default_text_size
//     }

//     fn measure(&self, contents: &str, size: f32, font: Font, bounds: Size) -> (f32, f32) {
//         self.text_pipeline.measure(contents, size, font, bounds)
//     }

//     fn hit_test(
//         &self,
//         contents: &str,
//         size: f32,
//         font: Font,
//         bounds: Size,
//         point: iced_native::Point,
//         nearest_only: bool,
//     ) -> Option<text::Hit> {
//         self.text_pipeline
//             .hit_test(contents, size, font, bounds, point, nearest_only)
//     }
// }

// #[cfg(feature = "image_rs")]
// impl backend::Image for Backend {
//     fn dimensions(&self, handle: &iced_native::image::Handle) -> (u32, u32) {
//         self.image_pipeline.dimensions(handle)
//     }
// }

// #[cfg(feature = "svg")]
// impl backend::Svg for Backend {
//     fn viewport_dimensions(&self, handle: &iced_native::svg::Handle) -> (u32, u32) {
//         self.image_pipeline.viewport_dimensions(handle)
//     }
// }
