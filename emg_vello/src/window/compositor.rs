use std::time::Instant;

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use vello::{
    util::{DeviceHandle, RenderContext as VelloRenderContext, RenderSurface},
    RenderParams, RendererOptions, Scene, SceneBuilder,
};

use emg_graphics_backend::{window::compositor as compositor_arch, Error};
use emg_native::futures;
use tracing::{debug_span, info, instrument};

use crate::{scenes::SimpleText, Backend, Renderer, SceneFrag, Settings};

use super::stats;
// ────────────────────────────────────────────────────────────────────────────────
#[cfg(feature = "show-fps")]
use static_init::dynamic;

#[cfg(feature = "show-fps")]
#[dynamic]
static mut FRAME_START_TIME: std::time::Instant = Instant::now();

// ─────────────────────────────────────────────────────────────────────────────

/// A window graphics backend for  `vello`.
#[allow(missing_debug_implementations)]
pub struct Compositor {
    settings: Settings,
    render_cx: VelloRenderContext,
    scene: Scene,
    surface: RenderSurface,
    render_params: RenderParams,
    simple_text: SimpleText,
    #[cfg(feature = "show-fps")]
    stats: stats::Stats,
}

impl Compositor {
    /// Requests a new [`Compositor`] with the given [`Settings`].
    ///
    /// Returns `None` if no compatible graphics adapter could be found.
    // #[instrument(skip(window))]
    pub async fn request<W>(settings: Settings, window: &W) -> Result<Self, Error>
    where
        W: HasRawWindowHandle + HasRawDisplayHandle,
    {
        // let instance = Instance::new(settings.flags)?;
        let mut render_cx =
            VelloRenderContext::new().map_err(|e| Error::BackendError(e.to_string()))?;

        let scene = Scene::new();

        let vp_scale_factor = settings.vp_scale_factor.unwrap();

        debug_span!(
            "window_size",
            "======== create_surface settings size: {} {} ,size * vp_scale_factor= {} {}",
            settings.width,
            settings.height,
            (settings.width as f64 * vp_scale_factor).round() as u32,
            (settings.height as f64 * vp_scale_factor).round() as u32,
        )
        .in_scope(|| {});

        let mut surface = render_cx
            .create_surface(
                &window, //NOTE 物理尺寸
                (settings.width as f64 * vp_scale_factor).round() as u32,
                (settings.height as f64 * vp_scale_factor).round() as u32,
                // (settings.width as f64) as u32,
                // (settings.height as f64) as u32,
            )
            .await;

        let render_params = RenderParams {
            base_color: vello::peniko::Color::BLACK,
            width: surface.config.width,
            height: surface.config.height,
        };

        // ─────────────────────────────────────────────────────────────────────────────
        let simple_text = crate::scenes::SimpleText::new();

        // render_cx.set_present_mode(&mut surface, wgpu::PresentMode::AutoNoVsync);
        render_cx.set_present_mode(&mut surface, wgpu::PresentMode::AutoVsync);

        // ─────────────────────────────────────────────────────────────────────────────
        #[cfg(feature = "show-fps")]
        let stats = stats::Stats::new();
        // ─────────────────────────────────────────────────────────────

        Ok(Compositor {
            settings,
            render_cx,
            scene,
            surface,
            render_params,
            simple_text,
            #[cfg(feature = "show-fps")]
            stats,
        })
    }

    pub fn device_handle(&self) -> &DeviceHandle {
        &self.render_cx.devices[self.surface.dev_id]
    }

    /// Creates a new rendering [`Backend`] for this [`Compositor`].
    pub fn create_backend(&self) -> Result<Backend, Error> {
        Backend::new(
            self.device_handle(),
            &RendererOptions {
                surface_format: Some(self.surface.format),
            },
        )
    }
}

impl compositor_arch::Compositor for Compositor {
    type Settings = Settings;
    type Renderer = Renderer;
    // type Surface = Option<()>;
    type Surface = ();

    #[instrument(skip(window), name = "Compositor::new")]
    fn new<W>(settings: Self::Settings, window: &W) -> Result<(Self, Self::Renderer), Error>
    where
        W: HasRawWindowHandle + HasRawDisplayHandle,
    {
        info!("Compositor new \n\t gpu settings:{:#?}", &settings);
        let compositor = futures::executor::block_on(Self::request(settings, window))?;

        let backend = compositor.create_backend()?;

        Ok((compositor, Renderer::new(backend)))
    }

    fn create_surface<W>(&mut self, _window: &W) -> Self::Surface
    where
        W: HasRawWindowHandle + HasRawDisplayHandle,
    {
    }

    fn configure_surface(&mut self, _surface: &mut Self::Surface, width: u32, height: u32) {
        let _span =
            debug_span!(target: "resize","Compositor::configure_surface", ?width,?height).entered();

        //NOTE if base_color change , need reset base_color
        self.render_params.width = width;
        self.render_params.height = height;

        self.render_cx
            .resize_surface(&mut self.surface, width, height);
        //NOTE no need request_redraw because it will Redraw immediately
    }

    fn fetch_information(&self) -> compositor_arch::Information {
        // let information = self.adapter.get_info();

        // compositor_arch::Information {
        //     adapter: information.name,
        //     backend: format!("{:?}", information.backend),
        // }
        todo!()
    }

    fn present(
        &mut self,
        renderer: &mut Renderer,
        scene_ctx: &SceneFrag,
        _surface: &mut Self::Surface,
    ) -> Result<(), compositor_arch::SurfaceError> {
        let backend = renderer.backend_mut();
        let mut sb = SceneBuilder::for_scene(&mut self.scene);
        sb.append(&scene_ctx.0, scene_ctx.1);

        #[cfg(feature = "show-fps")]
        {
            let snapshot = self.stats.snapshot();
            let stats_shown = true;
            let vsync_on = true;

            // ─────────────────────────────────────────────────────────────────────────────

            if stats_shown {
                snapshot.draw_layer(
                    &mut sb,
                    &mut self.simple_text,
                    self.render_params.width as f64,
                    self.render_params.height as f64,
                    self.stats.samples(),
                    vsync_on,
                );
            }
        }

        // render_cx: &VelloRenderContext,
        // scene: &Scene,
        // surface: &RenderSurface,
        backend.present(
            self.device_handle(),
            &self.scene,
            &self.surface,
            &self.render_params,
        );

        #[cfg(feature = "show-fps")]
        {
            let new_time = Instant::now();

            self.stats.add_sample(stats::Sample {
                frame_time_us: (new_time - *FRAME_START_TIME.read()).as_micros() as u64,
            });
            let mut w = FRAME_START_TIME.write();
            *w = new_time;
        }

        Ok(())
    }
}
