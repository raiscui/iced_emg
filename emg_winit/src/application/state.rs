use crate::conversion;
use crate::{Application, Debug, Mode, Viewport};
use emg_common::{na, Pos};
use emg_graphics_backend::window::Compositor;
use emg_native::G_POS;
use emg_state::{CloneStateVar, StateAnchor, StateMultiAnchor, StateVar};
use std::{cell::Cell, marker::PhantomData, rc::Rc};
use winit::event::{Touch, WindowEvent};
use winit::window::Window;

/// The state of a windowed [`Application`].
#[allow(missing_debug_implementations)]
pub struct State<A: Application> {
    title: String,
    mode: Mode,
    scale_factor: f64,
    viewport: Viewport,
    viewport_version: usize,
    cursor_position: StateAnchor<Option<Pos>>,
    modifiers: winit::event::ModifiersState,
    application: PhantomData<A>,
}

impl<A: Application> State<A> {
    /// Creates a new [`State`] for the provided [`Application`] and window.
    pub fn new(application: &A, window: &Window) -> Self {
        let title = application.title();
        let mode = application.mode();
        let scale_factor = application.scale_factor();
        // let theme = application.theme();
        // let appearance = theme.appearance(application.style());

        let viewport = {
            let physical_size = window.inner_size();

            Viewport::new(
                na::Vector2::<u32>::new(physical_size.width, physical_size.height),
                window.scale_factor() * scale_factor,
            )
        };
        let cursor_position = {
            let scale_factor_rc = viewport.scale_factor_rc();

            G_POS.watch().map(move |opt_pos| {
                opt_pos
                    .as_ref()
                    .map(|pos| conversion::cursor_na_position(pos, scale_factor_rc.get()))
            })
        };

        Self {
            title,
            mode,
            scale_factor,
            viewport,
            viewport_version: 0,
            // TODO: Encode cursor availability in the type-system
            cursor_position,
            modifiers: winit::event::ModifiersState::default(),
            application: PhantomData,
        }
    }

    /// Returns the current [`Viewport`] of the [`State`].
    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    /// Returns the version of the [`Viewport`] of the [`State`].
    ///
    /// The version is incremented every time the [`Viewport`] changes.
    pub fn viewport_version(&self) -> usize {
        self.viewport_version
    }

    /// Returns the physical [`Size`] of the [`Viewport`] of the [`State`].
    pub fn physical_size(&self) -> na::Vector2<u32> {
        self.viewport.physical_size()
    }

    /// Returns the logical [`Size`] of the [`Viewport`] of the [`State`].
    pub fn logical_size(&self) -> na::Vector2<f32> {
        self.viewport.logical_size()
    }

    /// Returns the current scale factor of the [`Viewport`] of the [`State`].
    pub fn scale_factor(&self) -> f64 {
        self.viewport.scale_factor()
    }
    pub fn scale_factor_rc(&self) -> Rc<Cell<f64>> {
        self.viewport.scale_factor_rc()
    }

    // /// Returns the current cursor position of the [`State`].
    // pub fn cursor_position(&self) -> Pos {
    //     conversion::cursor_position(self.cursor_position, self.viewport.scale_factor())
    // }

    /// Returns the current keyboard modifiers of the [`State`].
    pub fn modifiers(&self) -> winit::event::ModifiersState {
        self.modifiers
    }

    // /// Returns the current theme of the [`State`].
    // pub fn theme(&self) -> &<A::Renderer as crate::Renderer>::Theme {
    //     &self.theme
    // }

    // /// Returns the current background [`Color`] of the [`State`].
    // pub fn background_color(&self) -> Color {
    //     self.appearance.background_color
    // }

    // /// Returns the current text [`Color`] of the [`State`].
    // pub fn text_color(&self) -> Color {
    //     self.appearance.text_color
    // }

    /// Processes the provided window event and updates the [`State`]
    /// accordingly.
    pub fn update(&mut self, window: &Window, event: &WindowEvent<'_>, _debug: &mut Debug) {
        match event {
            WindowEvent::Resized(new_size) => {
                let size = na::Vector2::<u32>::new(new_size.width, new_size.height);

                self.viewport = self
                    .viewport
                    .with_physical_size(size, window.scale_factor() * self.scale_factor);

                self.viewport_version = self.viewport_version.wrapping_add(1);
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor: new_scale_factor,
                new_inner_size,
            } => {
                let size = na::Vector2::<u32>::new(new_inner_size.width, new_inner_size.height);

                self.viewport = self
                    .viewport
                    .with_physical_size(size, new_scale_factor * self.scale_factor);

                self.viewport_version = self.viewport_version.wrapping_add(1);
            }
            WindowEvent::CursorMoved { position, .. } => {
                G_POS.set(Some(Pos::<f64>::new(position.x, position.y)))
            }
            WindowEvent::Touch(Touch {
                phase,
                location: position,
                ..
            }) => {
                //NOTE current never here at my air m2
                let _span = tracing::debug_span!("Touch", phase = ?phase).entered();
                G_POS.set(Some(Pos::<f64>::new(position.x, position.y)));
            }
            WindowEvent::CursorLeft { .. } => {
                // TODO: Encode cursor availability in the type-system
                // self.cursor_position = winit::dpi::PhysicalPosition::new(-1.0, -1.0);
                G_POS.set(None);
            }
            WindowEvent::ModifiersChanged(new_modifiers) => {
                self.modifiers = *new_modifiers;
            }
            #[cfg(feature = "debug")]
            WindowEvent::KeyboardInput {
                input:
                    winit::event::KeyboardInput {
                        virtual_keycode: Some(winit::event::VirtualKeyCode::F12),
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                ..
            } => _debug.toggle(),
            _ => {}
        }
    }

    /// Synchronizes the [`State`] with its [`Application`] and its respective
    /// window.
    ///
    /// Normally an [`Application`] should be synchronized with its [`State`]
    /// and window after calling [`Application::update`].
    ///
    /// [`Application::update`]: crate::Program::update
    //TODO use it
    pub fn synchronize(&mut self, application: &A, window: &Window) {
        // Update window title
        let new_title = application.title();

        if self.title != new_title {
            window.set_title(&new_title);

            self.title = new_title;
        }

        // Update window mode
        let new_mode = application.mode();

        if self.mode != new_mode {
            window.set_fullscreen(conversion::fullscreen(window.current_monitor(), new_mode));

            window.set_visible(conversion::visible(new_mode));

            self.mode = new_mode;
        }

        // Update scale factor
        let new_scale_factor = application.scale_factor();

        if self.scale_factor != new_scale_factor {
            let size = window.inner_size();

            self.viewport = self.viewport.with_physical_size(
                na::Vector2::new(size.width, size.height),
                window.scale_factor() * new_scale_factor,
            );

            self.scale_factor = new_scale_factor;
        }

        // Update theme and appearance
        // self.theme = application.theme();
        // self.appearance = self.theme.appearance(application.style());
    }

    pub fn cursor_position(&self) -> &StateAnchor<Option<Pos>> {
        &self.cursor_position
    }
}
