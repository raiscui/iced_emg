use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use vello::{
    util::{DeviceHandle, RenderContext as VelloRenderContext, RenderSurface},
    Scene, SceneBuilder,
};

use emg_graphics_backend::{window::compositor as compositor_arch, Error};
use emg_native::{futures, DPR};
use tracing::{info, instrument};

use crate::{Backend, Renderer, SceneFrag, Settings};
// ────────────────────────────────────────────────────────────────────────────────

/// A window graphics backend for  `vello`.
#[allow(missing_debug_implementations)]
pub struct Compositor {
    settings: Settings,
    render_cx: VelloRenderContext,
    scene: Scene,
    surface: RenderSurface,
}

impl Compositor {
    /// Requests a new [`Compositor`] with the given [`Settings`].
    ///
    /// Returns `None` if no compatible graphics adapter could be found.
    #[instrument(skip(window))]
    pub async fn request<W>(settings: Settings, window: &W) -> Result<Self, Error>
    where
        W: HasRawWindowHandle + HasRawDisplayHandle,
    {
        // let instance = Instance::new(settings.flags)?;
        let mut render_cx =
            VelloRenderContext::new().map_err(|e| Error::BackendError(e.to_string()))?;

        let scene = Scene::new();

        info!(
            "======== create_surface settings size: {} {}",
            settings.width, settings.height
        );

        let surface = render_cx
            .create_surface(
                &window,
                //NOTE 物理尺寸
                (settings.width as f64 * DPR) as u32,
                (settings.height as f64 * DPR) as u32,
            )
            .await;

        Ok(Compositor {
            settings,
            render_cx,
            scene,
            surface,
        })
    }

    pub fn device_handle(&self) -> &DeviceHandle {
        &self.render_cx.devices[self.surface.dev_id]
    }

    /// Creates a new rendering [`Backend`] for this [`Compositor`].
    pub fn create_backend(&self) -> Result<Backend, Error> {
        Backend::new(self.device_handle())
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
        // let compositor = futures::executor::block_on(Self::request(settings))?;
        let compositor = futures::executor::block_on(Self::request(settings, window))?;

        let backend = compositor.create_backend()?;

        Ok((compositor, Renderer::new(backend)))
    }

    fn create_surface<W>(&mut self, _window: &W) -> Self::Surface
    where
        W: HasRawWindowHandle + HasRawDisplayHandle,
    {
        ()
    }

    fn configure_surface(&mut self, _surface: &mut Self::Surface, width: u32, height: u32) {
        self.render_cx
            .resize_surface(&mut self.surface, width, height);
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
        _surface: &mut Self::Surface,
    ) -> Result<(), compositor_arch::SurfaceError> {
        let backend = renderer.backend_mut();
        let mut sb = SceneBuilder::for_scene(&mut self.scene);
        sb.append(&scene_ctx.0, scene_ctx.1);

        // render_cx: &VelloRenderContext,
        // scene: &Scene,
        // surface: &RenderSurface,
        backend.present(self.device_handle(), &self.scene, &self.surface);
        Ok(())
    }
}
