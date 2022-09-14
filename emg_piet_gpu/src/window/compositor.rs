use futures::stream::StreamExt;

use emg_graphics_backend::{window::compositor as compositor_arch, Error};
use emg_native::futures;
use piet_gpu_hal::{Device, ImageLayout, Instance, Session, Surface};
use raw_window_handle::HasRawWindowHandle;
use tracing::{info, instrument};

use crate::{Backend, RenderCtx, Renderer, Settings, NUM_FRAMES};
// ────────────────────────────────────────────────────────────────────────────────

/// A window graphics backend for iced powered by `wgpu`.
#[allow(missing_debug_implementations)]
pub struct Compositor {
    settings: Settings,
    instance: Instance,
    surface: Surface,
    // swapchain: Swapchain,
    // adapter: wgpu::Adapter,
    // queue: wgpu::Queue,
    // staging_belt: wgpu::util::StagingBelt,
    // format: wgpu::TextureFormat,
    // theme: PhantomData<Theme>,
}

impl Compositor {
    /// Requests a new [`Compositor`] with the given [`Settings`].
    ///
    /// Returns `None` if no compatible graphics adapter could be found.
    #[instrument(skip(window))]
    pub async fn request<W: HasRawWindowHandle>(
        settings: Settings,
        window: &W,
    ) -> Result<Self, Error> {
        let instance = Instance::new(settings.flags)?;

        unsafe {
            // let surface = instance.surface(window)?;

            let surface = instance.surface(&window)?;

            // let swapchain =
            //     instance.swapchain(settings.width / 2, settings.height / 2, &device, &surface)?;

            Ok(Compositor {
                instance,
                surface,
                // session,
                // swapchain,
                settings,
                // adapter,
                // device,
                // queue,
                // staging_belt,
                // format,
                // theme: PhantomData,
            })
        }
    }

    /// Creates a new rendering [`Backend`] for this [`Compositor`].
    pub fn create_backend(&self) -> Result<Backend, Error> {
        let device = unsafe { self.instance.device()? };

        let swapchain = unsafe {
            self.instance.swapchain(
                //TODO use state.viewport
                self.settings.width,
                self.settings.height,
                &device,
                &self.surface,
            )?
        };

        let session = Session::new(device);

        Backend::new(swapchain, session, self.settings)
    }
}

impl emg_graphics_backend::window::Compositor for Compositor {
    type Settings = Settings;
    type Renderer = Renderer;
    type Surface = Option<()>;

    #[instrument(skip(window), name = "Compositor::new")]
    fn new<W: HasRawWindowHandle>(
        settings: Self::Settings,
        window: &W,
    ) -> Result<(Self, Self::Renderer), Error> {
        info!("Compositor new \n\t gpu settings:{:#?}", &settings);
        let compositor = futures::executor::block_on(Self::request(settings, window))?;

        let backend = compositor.create_backend()?;

        Ok((compositor, Renderer::new(backend)))
    }

    fn create_surface<W: HasRawWindowHandle>(&mut self, window: &W) -> Self::Surface {
        // #[allow(unsafe_code)]
        // unsafe {
        //     self.instance.surface(window)?
        // }
        //NOTE "piet gpu not need surface instance"
        None
    }

    fn configure_surface(&mut self, surface: &mut Self::Surface, width: u32, height: u32) {
        todo!()
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
        render_ctx: &mut RenderCtx,
        _surface: &mut Option<()>,
        // viewport: &Viewport,
        // background_color: Color,
        // overlay: &[T],
    ) -> Result<(), compositor_arch::SurfaceError> {
        let backend = renderer.backend_mut();
        backend.present(render_ctx);
        Ok(())
    }
}
