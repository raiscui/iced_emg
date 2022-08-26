/*
 * @Author: Rais
 * @Date: 2022-08-14 15:29:14
 * @LastEditTime: 2022-08-26 17:48:00
 * @LastEditors: Rais
 * @Description:
 */

use crate::{Settings, NUM_FRAMES};

use emg_graphics_backend::Error;

use piet_gpu::{RenderDriver, Renderer};
use piet_gpu_hal::{Device, ImageLayout, Semaphore, Session, Swapchain};
use tracing::debug;

// #[cfg(any(feature = "image_rs", feature = "svg"))]
// use crate::image;

/// A [`wgpu`] graphics backend for [`iced`].
///
/// [`wgpu`]: https://github.com/gfx-rs/wgpu-rs
/// [`iced`]: https://github.com/iced-rs/iced
pub struct Backend {
    swapchain: Swapchain,
    session: Session,
    render_driver: RenderDriver,
    present_semaphores: Vec<Semaphore>,
    current_frame: usize,
}

impl Backend {
    /// Creates a new [`Backend`].
    pub fn new(swapchain: Swapchain, session: Session, settings: Settings) -> Result<Self, Error> {
        unsafe {
            let present_semaphores = (0..NUM_FRAMES)
                .map(|_| session.create_semaphore())
                .collect::<Result<Vec<_>, Box<dyn std::error::Error + Send + Sync>>>()?;

            let renderer = Renderer::new(
                &session,
                settings.width * 2,
                settings.height * 2,
                NUM_FRAMES,
            )?;
            let render_driver = RenderDriver::new(&session, NUM_FRAMES, renderer);

            Ok(Self {
                swapchain,
                session,
                render_driver,
                present_semaphores,
                current_frame: 0,
            })
        }
    }

    pub fn wait_all(&mut self) {
        self.render_driver.wait_all(&self.session);
    }

    /// Draws the provided primitives in the given `TextureView`.
    ///
    /// The text provided as overlay will be rendered on top of the primitives.
    /// This is useful for rendering debug information.
    pub fn present(
        &mut self,
        render_ctx: &mut crate::RenderCtx,
        // device: &wgpu::Device,
        // staging_belt: &mut wgpu::util::StagingBelt,
        // encoder: &mut wgpu::CommandEncoder,
        // frame: &wgpu::TextureView,
        // primitives: &[Primitive],
        // viewport: &Viewport,
        // overlay_text: &[T],
    ) {
        debug!("Drawing");
        let session = &self.session;

        let frame_idx = self.current_frame % NUM_FRAMES;

        if self.current_frame >= NUM_FRAMES {
            let stats = unsafe { self.render_driver.get_timing_stats(session, frame_idx) };
            //TODO impl info_string
            // info_string = stats.short_summary();
        }

        if let Err(e) = self.render_driver.upload_render_ctx(session, render_ctx) {
            println!("error in uploading: {}", e);
        }

        let swapchain = &mut self.swapchain;
        let (image_idx, acquisition_semaphore) = unsafe { swapchain.next().unwrap() };
        let swap_image = unsafe { swapchain.image(image_idx) };

        self.render_driver.run_coarse(session).unwrap();
        let target = self.render_driver.record_fine(session).unwrap();
        let cmd_buf = target.cmd_buf;

        // Image -> Swapchain
        unsafe {
            cmd_buf.image_barrier(&swap_image, ImageLayout::Undefined, ImageLayout::BlitDst);
            cmd_buf.blit_image(target.image, &swap_image);
            cmd_buf.image_barrier(&swap_image, ImageLayout::BlitDst, ImageLayout::Present);

            self.render_driver
                .submit(
                    session,
                    &[&acquisition_semaphore],
                    &[&self.present_semaphores[frame_idx]],
                )
                .unwrap();

            swapchain
                .present(image_idx, &[&self.present_semaphores[frame_idx]])
                .unwrap();
        }

        self.render_driver.next_buffer();
        self.current_frame += 1;
    }

    // fn flush(
    //     &mut self,
    //     device: &wgpu::Device,
    //     scale_factor: f32,
    //     transformation: Transformation,
    //     layer: &Layer<'_>,
    //     staging_belt: &mut wgpu::util::StagingBelt,
    //     encoder: &mut wgpu::CommandEncoder,
    //     target: &wgpu::TextureView,
    //     target_width: u32,
    //     target_height: u32,
    // ) {
    //     let bounds = (layer.bounds * scale_factor).snap();

    //     if bounds.width < 1 || bounds.height < 1 {
    //         return;
    //     }

    //     if !layer.quads.is_empty() {
    //         self.quad_pipeline.draw(
    //             device,
    //             staging_belt,
    //             encoder,
    //             &layer.quads,
    //             transformation,
    //             scale_factor,
    //             bounds,
    //             target,
    //         );
    //     }

    //     if !layer.meshes.is_empty() {
    //         let scaled = transformation * Transformation::scale(scale_factor, scale_factor);

    //         self.triangle_pipeline.draw(
    //             device,
    //             staging_belt,
    //             encoder,
    //             target,
    //             target_width,
    //             target_height,
    //             scaled,
    //             scale_factor,
    //             &layer.meshes,
    //         );
    //     }

    //     #[cfg(any(feature = "image_rs", feature = "svg"))]
    //     {
    //         if !layer.images.is_empty() {
    //             let scaled = transformation * Transformation::scale(scale_factor, scale_factor);

    //             self.image_pipeline.draw(
    //                 device,
    //                 staging_belt,
    //                 encoder,
    //                 &layer.images,
    //                 scaled,
    //                 bounds,
    //                 target,
    //                 scale_factor,
    //             );
    //         }
    //     }

    //     if !layer.text.is_empty() {
    //         for text in layer.text.iter() {
    //             // Target physical coordinates directly to avoid blurry text
    //             let text = wgpu_glyph::Section {
    //                 // TODO: We `round` here to avoid rerasterizing text when
    //                 // its position changes slightly. This can make text feel a
    //                 // bit "jumpy". We may be able to do better once we improve
    //                 // our text rendering/caching pipeline.
    //                 screen_position: (
    //                     (text.bounds.x * scale_factor).round(),
    //                     (text.bounds.y * scale_factor).round(),
    //                 ),
    //                 // TODO: Fix precision issues with some scale factors.
    //                 //
    //                 // The `ceil` here can cause some words to render on the
    //                 // same line when they should not.
    //                 //
    //                 // Ideally, `wgpu_glyph` should be able to compute layout
    //                 // using logical positions, and then apply the proper
    //                 // scaling when rendering. This would ensure that both
    //                 // measuring and rendering follow the same layout rules.
    //                 bounds: (
    //                     (text.bounds.width * scale_factor).ceil(),
    //                     (text.bounds.height * scale_factor).ceil(),
    //                 ),
    //                 text: vec![wgpu_glyph::Text {
    //                     text: text.content,
    //                     scale: wgpu_glyph::ab_glyph::PxScale {
    //                         x: text.size * scale_factor,
    //                         y: text.size * scale_factor,
    //                     },
    //                     font_id: self.text_pipeline.find_font(text.font),
    //                     extra: wgpu_glyph::Extra {
    //                         color: text.color,
    //                         z: 0.0,
    //                     },
    //                 }],
    //                 layout: wgpu_glyph::Layout::default()
    //                     .h_align(match text.horizontal_alignment {
    //                         alignment::Horizontal::Left => wgpu_glyph::HorizontalAlign::Left,
    //                         alignment::Horizontal::Center => wgpu_glyph::HorizontalAlign::Center,
    //                         alignment::Horizontal::Right => wgpu_glyph::HorizontalAlign::Right,
    //                     })
    //                     .v_align(match text.vertical_alignment {
    //                         alignment::Vertical::Top => wgpu_glyph::VerticalAlign::Top,
    //                         alignment::Vertical::Center => wgpu_glyph::VerticalAlign::Center,
    //                         alignment::Vertical::Bottom => wgpu_glyph::VerticalAlign::Bottom,
    //                     }),
    //             };

    //             self.text_pipeline.queue(text);
    //         }

    //         self.text_pipeline.draw_queued(
    //             device,
    //             staging_belt,
    //             encoder,
    //             target,
    //             transformation,
    //             wgpu_glyph::Region {
    //                 x: bounds.x,
    //                 y: bounds.y,
    //                 width: bounds.width,
    //                 height: bounds.height,
    //             },
    //         );
    //     }
    // }
}

impl emg_graphics_backend::Backend for Backend {
    type ImplRenderContext = crate::RenderCtx;

    fn on_loop_destroyed(&mut self) {
        self.wait_all();
    }

    fn new_render_ctx(&self) -> Self::ImplRenderContext {
        crate::RenderCtx::new()
    }

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
