/*
 * @Author: Rais
 * @Date: 2022-08-11 17:53:01
 * @LastEditTime: 2023-04-25 22:48:43
 * @LastEditors: Rais
 * @Description:
 */
//! Configure your application.
use crate::window;

/// The settings of an application.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct Settings<Flags> {
    /// The identifier of the application.
    ///
    /// If provided, this identifier may be used to identify the application or
    /// communicate with it through the windowing system.
    pub id: Option<String>,

    /// The window settings.
    ///
    /// They will be ignored on the Web.
    pub window: window::Settings,

    /// The data needed to initialize the [`Application`].
    ///
    /// [`Application`]: crate::Application
    pub flags: Flags,

    /// The bytes of the font that will be used by default.
    ///
    /// If `None` is provided, a default system font will be chosen.
    // TODO: Add `name` for web compatibility
    pub default_font: Option<&'static [u8]>,

    /// The text size that will be used by default.
    ///
    /// The default value is 20.
    pub default_text_size: u16,

    /// If enabled, spread text workload in multiple threads when multiple cores
    /// are available.
    ///
    /// By default, it is disabled.
    pub text_multithreading: bool,

    /// If set to true, the renderer will try to perform antialiasing for some
    /// primitives.
    ///
    /// Enabling it can produce a smoother result in some widgets, like the
    /// [`Canvas`], at a performance cost.
    ///
    /// By default, it is disabled.
    ///
    /// [`Canvas`]: crate::widget::Canvas
    pub antialiasing: bool,

    /// Whether the [`Application`] should exit when the user requests the
    /// window to close (e.g. the user presses the close button).
    ///
    /// By default, it is enabled.
    ///
    /// [`Application`]: crate::Application
    pub exit_on_close_request: bool,

    pub vsync: bool,
}

impl<Flags> Settings<Flags> {
    /// Initialize [`Application`] settings using the given data.
    ///
    /// [`Application`]: crate::Application
    pub fn with_flags(flags: Flags) -> Self {
        let default_settings = Settings::<()>::default();

        Self {
            flags,
            id: default_settings.id,
            window: default_settings.window,
            default_font: default_settings.default_font,
            default_text_size: default_settings.default_text_size,
            text_multithreading: default_settings.text_multithreading,
            antialiasing: default_settings.antialiasing,
            exit_on_close_request: default_settings.exit_on_close_request,
            vsync: default_settings.vsync,
        }
    }
}

impl<Flags> Default for Settings<Flags>
where
    Flags: Default,
{
    fn default() -> Self {
        Self {
            id: None,
            window: window::Settings::default(),
            flags: Default::default(),
            default_font: std::option::Option::default(),
            default_text_size: 20,
            text_multithreading: false,
            antialiasing: false,
            exit_on_close_request: true,
            vsync: true,
        }
    }
}

#[cfg(all(feature = "gpu"))]
impl<Flags> From<Settings<Flags>> for emg_winit::Settings<Flags> {
    fn from(settings: Settings<Flags>) -> Self {
        Self {
            id: settings.id,
            window: settings.window.into(),
            flags: settings.flags,
            exit_on_close_request: settings.exit_on_close_request,
        }
    }
}
