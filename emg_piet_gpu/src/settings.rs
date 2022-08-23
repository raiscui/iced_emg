/*
 * @Author: Rais
 * @Date: 2022-08-14 00:09:14
 * @LastEditTime: 2022-08-23 16:00:06
 * @LastEditors: Rais
 * @Description:
 */
//! Configure a renderer.

use piet_gpu_hal::InstanceFlags;

/// The settings of a [`Backend`].
///
/// [`Backend`]: crate::Backend
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    pub width: usize,
    pub height: usize,
    pub flags: InstanceFlags,
    // /// The present mode of the [`Backend`].
    // ///
    // /// [`Backend`]: crate::Backend
    // pub present_mode: wgpu::PresentMode,

    // /// The internal graphics backend to use.
    // pub internal_backend: wgpu::Backends,

    // /// The bytes of the font that will be used by default.
    // ///
    // /// If `None` is provided, a default system font will be chosen.
    // pub default_font: Option<&'static [u8]>,

    // /// The default size of text.
    // ///
    // /// By default, it will be set to 20.
    // pub default_text_size: u16,

    // /// If enabled, spread text workload in multiple threads when multiple cores
    // /// are available.
    // ///
    // /// By default, it is disabled.
    // pub text_multithreading: bool,

    // /// The antialiasing strategy that will be used for triangle primitives.
    // ///
    // /// By default, it is `None`.
    // pub antialiasing: Option<Antialiasing>,
}

impl Settings {
    /// Creates new [`Settings`] using environment configuration.
    ///
    /// Specifically:
    ///
    /// - The `internal_backend` can be configured using the `WGPU_BACKEND`
    /// environment variable. If the variable is not set, the primary backend
    /// will be used. The following values are allowed:
    ///     - `vulkan`
    ///     - `metal`
    ///     - `dx12`
    ///     - `dx11`
    ///     - `gl`
    ///     - `webgpu`
    ///     - `primary`
    pub fn from_env() -> Self {
        Settings { ..Self::default() }
    }
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            width: 1920,
            height: 1080,
            flags: InstanceFlags::default(),
            // present_mode: wgpu::PresentMode::AutoVsync,
            // internal_backend: wgpu::Backends::all(),
            // default_font: None,
            // default_text_size: 20,
            // text_multithreading: false,
            // antialiasing: None,
        }
    }
}

// fn backend_from_env() -> Option<wgpu::Backends> {
//     std::env::var("WGPU_BACKEND")
//         .ok()
//         .map(|backend| match backend.to_lowercase().as_str() {
//             "vulkan" => wgpu::Backends::VULKAN,
//             "metal" => wgpu::Backends::METAL,
//             "dx12" => wgpu::Backends::DX12,
//             "dx11" => wgpu::Backends::DX11,
//             "gl" => wgpu::Backends::GL,
//             "webgpu" => wgpu::Backends::BROWSER_WEBGPU,
//             "primary" => wgpu::Backends::PRIMARY,
//             other => panic!("Unknown backend: {}", other),
//         })
// }
