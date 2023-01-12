/*
 * @Author: Rais
 * @Date: 2022-08-13 16:06:48
 * @LastEditTime: 2023-01-12 15:13:39
 * @LastEditors: Rais
 * @Description:
 */
//! A compositor is responsible for initializing a renderer and managing window
//! surfaces.

use emg_native::renderer::Renderer;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use thiserror::Error as TError;

use crate::Error;

/// A graphics compositor that can draw to windows.
pub trait Compositor: Sized {
    /// The settings of the backend.
    type Settings: Default;

    /// The iced renderer of the backend.
    type Renderer: Renderer;

    /// The surface of the backend.
    type Surface;

    /// Creates a new [`Compositor`].
    fn new<W>(settings: Self::Settings, window: &W) -> Result<(Self, Self::Renderer), Error>
    where
        W: HasRawWindowHandle + HasRawDisplayHandle;

    /// Crates a new [`Surface`] for the given window.
    ///
    /// [`Surface`]: Self::Surface
    fn create_surface<W>(&mut self, window: &W) -> Self::Surface
    where
        W: HasRawWindowHandle + HasRawDisplayHandle;

    /// Configures a new [`Surface`] with the given dimensions.
    ///
    /// [`Surface`]: Self::Surface
    fn configure_surface(&mut self, surface: &mut Self::Surface, width: u32, height: u32);

    /// Returns [`GraphicsInformation`] used by this [`Compositor`].
    fn fetch_information(&self) -> Information;

    /// Presents the [`Renderer`] primitives to the next frame of the given [`Surface`].
    ///
    /// [`Renderer`]: Self::Renderer
    /// [`Surface`]: Self::Surface
    fn present(
        &mut self,
        renderer: &mut Self::Renderer,
        scene_ctx: &<Self::Renderer as Renderer>::SceneCtx,
        surface: &mut Self::Surface,
        // viewport: &Viewport,
        // background_color: Color,
        // overlay: &[T],
    ) -> Result<(), SurfaceError>;
}

/// Result of an unsuccessful call to [`Compositor::present`].
#[derive(Clone, PartialEq, Eq, Debug, TError)]
pub enum SurfaceError {
    /// A timeout was encountered while trying to acquire the next frame.
    #[error("A timeout was encountered while trying to acquire the next frame")]
    Timeout,
    /// The underlying surface has changed, and therefore the surface must be updated.
    #[error("The underlying surface has changed, and therefore the surface must be updated.")]
    Outdated,
    /// The swap chain has been lost and needs to be recreated.
    #[error("The surface has been lost and needs to be recreated")]
    Lost,
    /// There is no more memory left to allocate a new frame.
    #[error("There is no more memory left to allocate a new frame")]
    OutOfMemory,
}

/// Contains information's about the graphics (e.g. graphics adapter, graphics backend).
#[derive(Debug)]
pub struct Information {
    /// Contains the graphics adapter.
    pub adapter: String,
    /// Contains the graphics backend.
    pub backend: String,
}
