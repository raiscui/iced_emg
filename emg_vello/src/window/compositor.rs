use futures::stream::StreamExt;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use vello::{
    util::{RenderContext as VelloRenderContext, RenderSurface},
    Scene, SceneBuilder,
};

use emg_graphics_backend::{window::compositor as compositor_arch, Error};
use emg_native::{futures, DPR};
use tracing::{info, instrument};

use crate::{Backend, Renderer, SceneFrag, Settings, NUM_FRAMES};
// ────────────────────────────────────────────────────────────────────────────────

/// A window graphics backend for  `vello`.
#[allow(missing_debug_implementations)]
pub struct Compositor {
    settings: Settings,
    render_cx: VelloRenderContext,
}

impl Compositor {
    /// Requests a new [`Compositor`] with the given [`Settings`].
    ///
    /// Returns `None` if no compatible graphics adapter could be found.
    #[instrument]
    pub async fn request(settings: Settings) -> Result<Self, Error> {
        // let instance = Instance::new(settings.flags)?;
        let render_cx = VelloRenderContext::new()
            .await
            .map_err(|e| Error::BackendError(e.to_string()))?;

        Ok(Compositor {
            settings,
            render_cx,
        })
    }

    /// Creates a new rendering [`Backend`] for this [`Compositor`].
    pub fn create_backend(&self) -> Result<Backend, Error> {
        Backend::new(&self.render_cx)
    }
}

impl compositor_arch::Compositor for Compositor {
    type Settings = Settings;
    type Renderer = Renderer;
    // type Surface = Option<()>;
    type Surface = RenderSurface;

    #[instrument(name = "Compositor::new")]
    fn new(settings: Self::Settings) -> Result<(Self, Self::Renderer), Error> {
        info!("Compositor new \n\t gpu settings:{:#?}", &settings);
        let compositor = futures::executor::block_on(Self::request(settings))?;

        let backend = compositor.create_backend()?;

        Ok((compositor, Renderer::new(backend)))
    }

    fn create_surface<W>(&mut self, window: &W) -> Self::Surface
    where
        W: HasRawWindowHandle + HasRawDisplayHandle,
    {
        info!(
            "======== create_surface settings size: {} {}",
            self.settings.width, self.settings.height
        );

        self.render_cx.create_surface(
            &window,
            //NOTE 物理尺寸
            (self.settings.width as f64 * DPR) as u32,
            (self.settings.height as f64 * DPR) as u32,
        )
    }

    fn configure_surface(&mut self, surface: &mut Self::Surface, width: u32, height: u32) {
        self.render_cx.resize_surface(surface, width, height);
        // window.request_redraw();
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
        surface: &mut RenderSurface,
    ) -> Result<(), compositor_arch::SurfaceError> {
        let backend = renderer.backend_mut();
        let mut sb = SceneBuilder::for_scene(&mut backend.scene);
        sb.append(&scene_ctx.0, scene_ctx.1);

        // render_cx: &VelloRenderContext,
        // scene: &Scene,
        // surface: &RenderSurface,
        backend.present(&self.render_cx, surface);
        Ok(())
    }
}
