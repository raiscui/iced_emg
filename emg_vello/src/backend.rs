/*
 * @Author: Rais
 * @Date: 2022-08-14 15:29:14
 * @LastEditTime: 2023-02-01 15:21:27
 * @LastEditors: Rais
 * @Description:
 */
use vello::{util::RenderSurface as VelloRenderSurface, Renderer as VelloRenderer};
use vello::{
    util::{block_on_wgpu, DeviceHandle},
    Scene,
};

use emg_graphics_backend::Error;

// #[cfg(any(feature = "image_rs", feature = "svg"))]
// use crate::image;

/// A [`wgpu`] graphics backend for [`elemg`].
pub struct Backend {
    renderer: VelloRenderer,
}

impl Backend {
    /// Creates a new [`Backend`].
    pub fn new(device_handle: &DeviceHandle) -> Result<Self, Error> {
        let renderer = VelloRenderer::new(&device_handle.device)
            .map_err(|e| Error::BackendError(e.to_string()))?;

        Ok(Self { renderer })
    }

    /// Draws the provided primitives in the given `TextureView`.
    pub fn present(
        &mut self,
        device_handle: &DeviceHandle,
        scene: &Scene,
        surface: &VelloRenderSurface,
    ) {
        let surface_texture = surface
            .surface
            .get_current_texture()
            .expect("failed to get surface texture");

        // self.renderer
        //     .render_to_surface(
        //         &device_handle.device,
        //         &device_handle.queue,
        //         scene,
        //         &surface_texture,
        //         surface.config.width,
        //         surface.config.height,
        //     )
        //     .expect("failed to render to surface");

        #[cfg(not(target_arch = "wasm32"))]
        {
            block_on_wgpu(
                &device_handle.device,
                self.renderer.render_to_surface_async(
                    &device_handle.device,
                    &device_handle.queue,
                    scene,
                    &surface_texture,
                    surface.config.width,
                    surface.config.height,
                ),
            )
            .expect("failed to render to surface");
        }
        // Note: in the wasm case, we're currently not running the robust
        // pipeline, as it requires more async wiring for the readback.
        #[cfg(target_arch = "wasm32")]
        renderer
            .render_to_surface(
                &device_handle.device,
                &device_handle.queue,
                scene,
                &surface_texture,
                surface.config.width,
                surface.config.height,
            )
            .expect("failed to render to surface");

        surface_texture.present();

        device_handle.device.poll(wgpu::Maintain::Poll);
    }
}

impl emg_graphics_backend::Backend for Backend {
    type SceneCtx = crate::SceneFrag;

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
