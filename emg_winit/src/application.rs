/*
 * @Author: Rais
 * @Date: 2022-08-13 13:11:58
 * @LastEditTime: 2023-01-12 15:29:17
 * @LastEditors: Rais
 * @Description:
 */
//! Create interactive, native cross-platform applications.
mod state;

use emg_common::{IdStr, Pos, Vector};
pub use state::State;
use winit::event_loop::EventLoopBuilder;

use crate::clipboard::{self, Clipboard};
use crate::conversion;
use crate::mouse;
use crate::{Command, Debug, Error, Executor, FutureRuntime, Mode, Proxy, Settings};
use emg_state::{use_state, use_state_impl::CloneStateVar, StateVar};

use emg_element::{GTreeBuilderFn, GraphProgram};
use emg_futures::futures;
use emg_futures::futures::channel::mpsc;
use emg_graphics_backend::window::{compositor, Compositor};
use emg_native::{
    event::{EventFlag, EventWithFlagType},
    program::Program,
    renderer::Renderer,
    Event,
};
use emg_state::CloneStateAnchor;
use tracing::{info, info_span, instrument};

// use emg_native::user_interface::{self, UserInterface};
// ────────────────────────────────────────────────────────────────────────────────

// ────────────────────────────────────────────────────────────────────────────────

/// An interactive, native cross-platform application.
///
/// This trait is the main entrypoint of Iced. Once implemented, you can run
/// your GUI application by simply calling [`run`]. It will run in
/// its own window.
///
/// An [`Application`] can execute asynchronous actions by returning a
/// [`Command`] in some of its methods.
///
/// When using an [`Application`] with the `debug` feature enabled, a debug view
/// can be toggled by pressing `F12`.
pub trait Application: GraphProgram {
    /// The data needed to initialize your [`Application`].

    type Flags;
    // type Renderer: Renderer<ImplRenderContext = Self::ImplRenderContext>;

    /// Initializes the [`Application`] with the flags provided to
    /// [`run`] as part of the [`Settings`].
    ///
    /// Here is where you should return the initial state of your app.
    ///
    /// Additionally, you can return a [`Command`] if you need to perform some
    /// async action in the background on startup. This is useful if you want to
    /// load state from a file, perform an initial HTTP request, etc.
    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>);

    /// Returns the current title of the [`Application`].
    ///
    /// This title can be dynamic! The runtime will automatically update the
    /// title of your application when necessary.
    fn title(&self) -> String;

    // /// Returns the current [`Theme`] of the [`Application`].
    // fn theme(&self) -> <Self::Renderer as crate::Renderer>::Theme;

    // /// Returns the [`Style`] variation of the [`Theme`].
    // fn style(&self) -> <<Self::Renderer as crate::Renderer>::Theme as StyleSheet>::Style {
    //     Default::default()
    // }

    // /// Returns the event `Subscription` for the current state of the
    // /// application.
    // ///
    // /// The messages produced by the `Subscription` will be handled by
    // /// [`update`](#tymethod.update).
    // ///
    // /// A `Subscription` will be kept alive as long as you keep returning it!
    // ///
    // /// By default, it returns an empty subscription.
    // fn subscription(&self) -> Subscription<Self::Message> {
    //     Subscription::none()
    // }

    /// Returns the current [`Application`] mode.
    ///
    /// The runtime will automatically transition your application if a new mode
    /// is returned.
    ///
    /// By default, an application will run in windowed mode.
    fn mode(&self) -> Mode {
        Mode::Windowed
    }

    /// Returns the scale factor of the [`Application`].
    ///
    /// It can be used to dynamically control the size of the UI at runtime
    /// (i.e. zooming).
    ///
    /// For instance, a scale factor of `2.0` will make widgets twice as big,
    /// while a scale factor of `0.5` will shrink them to half their size.
    ///
    /// By default, it returns `1.0`.
    fn scale_factor(&self) -> f64 {
        1.0
    }

    /// Returns whether the [`Application`] should be terminated.
    ///
    /// By default, it returns `false`.
    fn should_exit(&self) -> bool {
        false
    }
}

// #[must_use]
// pub fn global_cursor() -> StateVar<Pos<f64>> {
//     G_CURSOR.with(|sv| *sv)
// }

/// Runs an [`Application`] with an executor, compositor, and the provided
/// settings.
// #[instrument(skip_all, name = "winit->run")]
pub fn run<A, E, C>(
    settings: Settings<A::Flags>,
    compositor_settings: C::Settings,
) -> Result<(), crate::Error>
where
    A: Application + 'static,
    E: Executor + 'static,
    C: Compositor<Renderer = A::Renderer> + 'static,
{
    use futures::task;
    use futures::Future;
    use winit::event_loop::EventLoop;

    let mut debug = Debug::new();
    debug.startup_started();

    // let event_loop = EventLoop::with_user_event();
    let event_loop = EventLoopBuilder::with_user_event().build();
    let mut proxy = event_loop.create_proxy();

    let mut future_runtime = {
        let proxy = Proxy::new(event_loop.create_proxy());
        let executor = E::new().map_err(crate::Error::ExecutorCreationFailed)?;

        FutureRuntime::new(executor, proxy)
    };

    let (application, init_command) = {
        let flags = settings.flags;

        future_runtime.enter(|| A::new(flags))
    };

    let builder = settings.window.into_builder(
        &application.title(),
        application.mode(),
        event_loop.primary_monitor(),
        settings.id,
    );

    info!("Window builder: {:#?}", builder);

    let window = builder
        .build(&event_loop)
        .map_err(crate::Error::WindowCreationFailed)?;

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        let canvas = window.canvas();

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();

        let _ = body
            .append_child(&canvas)
            .expect("Append canvas to HTML body");
    }
    // ────────────────────────────────────────────────────────────────────────────────

    // let dpr = window.scale_factor();
    // let size: (f64, f64) = window.inner_size().to_logical::<f64>(dpr).into();
    // info!("Window size: {:?} {:?}", size, dpr);
    // emg_layout::global_width().set(size.0);
    // emg_layout::global_height().set(size.1);
    // ────────────────────────────────────────────────────────────────────────────────

    let mut clipboard = Clipboard::connect(&window);

    let (compositor, renderer) = C::new(compositor_settings, &window)?;

    run_command(
        init_command,
        &mut future_runtime,
        &mut clipboard,
        &mut proxy,
        &window,
        || compositor.fetch_information(),
    );
    // future_runtime.track(subscription);

    let (mut sender, receiver) = mpsc::unbounded();

    // let emg_graph = A::GraphType::default();
    // let root = application.tree_build();
    // let emg_graph_rc_refcell = Rc::new(RefCell::new(emg_graph));
    // emg_graph_rc_refcell.handle_root_in_topo(&root);
    let emg_graph_rc_refcell = application.graph_setup(&renderer);

    let mut instance = Box::pin(run_instance::<A, E, C>(
        application,
        compositor,
        renderer,
        future_runtime,
        clipboard,
        proxy,
        debug,
        receiver,
        window,
        settings.exit_on_close_request,
        emg_graph_rc_refcell,
    ));

    let mut context = task::Context::from_waker(task::noop_waker_ref());

    platform::run(event_loop, move |event, _, control_flow| {
        use winit::event_loop::ControlFlow;

        if let &mut ControlFlow::Exit = control_flow {
            return;
        }

        //just make ::ScaleFactorChanged to ::Resized
        let opt_event = match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. },
                window_id,
            } => Some(winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::Resized(*new_inner_size),
                window_id,
            }),
            _ => event.to_static(),
        };

        if let Some(event) = opt_event {
            sender.start_send(event).expect("Send event error");

            let poll = instance.as_mut().poll(&mut context);

            *control_flow = match poll {
                task::Poll::Pending => ControlFlow::Wait,
                task::Poll::Ready(_) => ControlFlow::Exit,
            };
        }
    })
}

#[instrument(skip_all)]
async fn run_instance<A, E, C>(
    mut application: A,
    mut compositor: C,
    mut renderer: A::Renderer,
    mut future_runtime: FutureRuntime<E, Proxy<A::Message>, A::Message>,
    mut clipboard: Clipboard,
    mut proxy: winit::event_loop::EventLoopProxy<A::Message>,
    mut debug: Debug,
    mut receiver: mpsc::UnboundedReceiver<winit::event::Event<'_, A::Message>>,
    window: winit::window::Window,
    exit_on_close_request: bool,
    g: A::GTreeBuilder,
) where
    A: Application + 'static,
    E: Executor + 'static,
    C: Compositor<Renderer = A::Renderer> + 'static,
{
    use emg_futures::futures::stream::StreamExt;
    use winit::event;

    info!(
        "======== will create_surface inner_size: {:?} ",
        window.inner_size()
    );

    let mut surface = compositor.create_surface(&window);
    let mut state = State::new(&application, &window);
    let mut viewport_version = state.viewport_version();

    // let physical_size = state.physical_size();

    // compositor.configure_surface(&mut surface, physical_size.width, physical_size.height);

    // let mut user_interface = ManuallyDrop::new(build_user_interface(
    //     &mut application,
    //     user_interface::Cache::default(),
    //     &mut renderer,
    //     state.logical_size(),
    //     &mut debug,
    // ));

    // let ctx = renderer.new_paint_ctx();
    //view
    let root_id = application.root_id();

    let native_events: StateVar<Vector<EventWithFlagType>> = use_state(Vector::new());
    let (event_matchs_sa, ctx_sa) =
        application.ctx(&g.graph(), &native_events.watch(), state.cursor_position());
    let mut ctx = ctx_sa.get();
    // let mut element = application.view(&g.graph());

    // window.request_redraw();

    //base node  = renderer.layout

    let mouse_interaction = mouse::Interaction::default();
    // let mut native_events: Vec<Event> = Vec::new();
    let native_events_is_empty = native_events.watch().map(|v| v.is_empty());
    let mut messages = Vec::new();

    debug.startup_finished();

    while let Some(winit_event) = receiver.next().await {
        // info!(target:"winit event", ?winit_event);

        match winit_event {
            event::Event::MainEventsCleared => {
                let _span = info_span!(target:"winit event","MainEventsCleared").entered();
                if native_events_is_empty.get() && messages.is_empty() {
                    continue;
                }
                //NOTE  has events or messages now -------------------

                debug.event_processing_started();
                info!(target:"winit event","native_events:{:?}", native_events);

                let event_matchs = event_matchs_sa.get();
                for ev in event_matchs.values().flat_map(|x| x.1.clone()) {
                    ev.call();
                }
                native_events.set(Vector::new());

                // let (interface_state, statuses) = user_interface.update(
                //     &events,
                //     state.cursor_position(),
                //     &mut renderer,
                //     &mut clipboard,
                //     &mut messages,
                // );

                //widget.on_event

                debug.event_processing_finished();

                // for event in events.drain(..).zip(statuses.into_iter()) {
                //     future_runtime.broadcast(event);
                // }

                if !messages.is_empty()
                // || matches!(interface_state, user_interface::State::Outdated,)
                {
                    // let cache = ManuallyDrop::into_inner(user_interface).into_cache();

                    // TODO Update application
                    // update(
                    //     &mut application,
                    //     &mut future_runtime,
                    //     &mut clipboard,
                    //     &mut proxy,
                    //     &mut debug,
                    //     &mut messages,
                    //     &window,
                    //     || compositor.fetch_information(),
                    // );

                    //TODO Update window
                    // state.synchronize(&application, &window);

                    let should_exit = application.should_exit();

                    // user_interface = ManuallyDrop::new(build_user_interface(
                    //     &mut application,
                    //     cache,
                    //     &mut renderer,
                    //     state.logical_size(),
                    //     &mut debug,
                    // ));

                    //TODO check rebuild need
                    // element = application.view(&g.graph());

                    if should_exit {
                        break;
                    }
                }

                debug.draw_started();
                // let new_mouse_interaction = user_interface.draw(
                //     &mut renderer,
                //     state.theme(),
                //     &renderer::Style {
                //         text_color: state.text_color(),
                //     },
                //     state.cursor_position(),
                // );
                info!(target:"winit event","element painting");
                // element.paint(&mut ctx);
                ctx = ctx_sa.get();

                debug.draw_finished();

                // if new_mouse_interaction != mouse_interaction {
                //     window.set_cursor_icon(conversion::mouse_interaction(new_mouse_interaction));

                //     mouse_interaction = new_mouse_interaction;
                // }

                window.request_redraw();
            }
            // event::Event::PlatformSpecific(event::PlatformSpecific::MacOS(
            //     event::MacOS::ReceivedUrl(url),
            // )) => {
            //     use emg_native::event;
            //     events.push(iced_native::Event::PlatformSpecific(
            //         event::PlatformSpecific::MacOS(event::MacOS::ReceivedUrl(url)),
            //     ));
            // }
            event::Event::UserEvent(message) => {
                // let _span = info_span!(target:"winit event","UserEvent").entered();
                info!(target:"winit event","UserEvent:{:?}",message);

                messages.push(message);
            }
            event::Event::RedrawRequested(_) => {
                // let _span = info_span!(target:"winit event","RedrawRequested").entered();
                info!(target:"winit event","RedrawRequested");

                // if physical_size.x == 0 || physical_size.y == 0 {
                //     continue;
                // }

                debug.render_started();
                let current_viewport_version = state.viewport_version();

                if viewport_version != current_viewport_version {
                    let physical_size = state.physical_size();

                    //     let logical_size = state.logical_size();

                    //     debug.layout_started();
                    //     user_interface = ManuallyDrop::new(
                    //         ManuallyDrop::into_inner(user_interface)
                    //             .relayout(logical_size, &mut renderer),
                    //     );
                    //     debug.layout_finished();

                    //     debug.draw_started();
                    //     let new_mouse_interaction = user_interface.draw(
                    //         &mut renderer,
                    //         state.theme(),
                    //         &renderer::Style {
                    //             text_color: state.text_color(),
                    //         },
                    //         state.cursor_position(),
                    //     );

                    //     if new_mouse_interaction != mouse_interaction {
                    //         window
                    //             .set_cursor_icon(conversion::mouse_interaction(new_mouse_interaction));

                    //         mouse_interaction = new_mouse_interaction;
                    //     }
                    //     debug.draw_finished();

                    compositor.configure_surface(&mut surface, physical_size.x, physical_size.y);

                    viewport_version = current_viewport_version;
                }
                match compositor.present(
                    &mut renderer,
                    &*ctx,
                    &mut surface,
                    // state.viewport(),
                    // state.background_color(),
                    // &debug.overlay(),
                ) {
                    Ok(()) => {
                        debug.render_finished();

                        // TODO: Handle animations!
                        // Maybe we can use `ControlFlow::WaitUntil` for this.
                    }
                    Err(error) => match error {
                        // This is an unrecoverable error.
                        compositor::SurfaceError::OutOfMemory => {
                            panic!("{error:?}");
                        }
                        _ => {
                            debug.render_finished();

                            // Try rendering again next frame.
                            window.request_redraw();
                        }
                    },
                }
            }
            event::Event::LoopDestroyed => {
                let _span = info_span!(target:"winit event","LoopDestroyed").entered();

                renderer.on_loop_destroyed();
            }

            event::Event::WindowEvent {
                event: window_event,
                ..
            } => {
                // let _span = info_span!(target:"winit event","WindowEvent").entered();
                info!(target:"winit event","WindowEvent:{:?}",window_event);
                // info!(target:"winit event","window.scale_factor():{}",window.scale_factor());//2

                if requests_exit(&window_event, state.modifiers()) && exit_on_close_request {
                    break;
                }

                state.update(&window, &window_event, &mut debug);

                if let Some(event_with_flag) =
                    conversion::window_event(&window_event, state.scale_factor(), state.modifiers())
                {
                    // native_events.push(event);
                    native_events.update(|ev| ev.push_back(event_with_flag));
                }

                if viewport_version != state.viewport_version() {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    // Manually drop the user interface
    //TODO check this need
    // drop(ManuallyDrop::into_inner(user_interface));
}

/// Returns true if the provided event should cause an [`Application`] to
/// exit.
pub fn requests_exit(
    event: &winit::event::WindowEvent<'_>,
    _modifiers: winit::event::ModifiersState,
) -> bool {
    use winit::event::WindowEvent;

    match event {
        WindowEvent::CloseRequested => true,
        #[cfg(target_os = "macos")]
        WindowEvent::KeyboardInput {
            input:
                winit::event::KeyboardInput {
                    virtual_keycode: Some(winit::event::VirtualKeyCode::Q),
                    state: winit::event::ElementState::Pressed,
                    ..
                },
            ..
        } if _modifiers.logo() => true,
        _ => false,
    }
}

// /// Builds a [`UserInterface`] for the provided [`Application`], logging
// /// [`struct@Debug`] information accordingly.
// pub fn build_user_interface<'a, A: Application>(
//     application: &'a mut A,
//     cache: user_interface::Cache,
//     renderer: &mut A::ImplRenderContext,
//     size: Size,
//     debug: &mut Debug,
// ) -> UserInterface<'a, A::Message, A::ImplRenderContext> {
//     debug.view_started();
//     let view = application.view();
//     debug.view_finished();

//     debug.layout_started();
//     let user_interface = UserInterface::build(view, size, cache, renderer);
//     debug.layout_finished();

//     user_interface
// }

//TODO check  where is  using? current not use
/// Updates an [`Application`] by feeding it the provided messages, spawning any
/// resulting [`Command`], and tracking its [`Subscription`].
pub fn update<A: Application, E: Executor>(
    application: &mut A,
    future_runtime: &mut FutureRuntime<E, Proxy<A::Message>, A::Message>,
    clipboard: &mut Clipboard,
    proxy: &mut winit::event_loop::EventLoopProxy<A::Message>,
    debug: &mut Debug,
    messages: &mut Vec<A::Message>,
    window: &winit::window::Window,
    graphics_info: impl FnOnce() -> compositor::Information + Copy,
) {
    for message in messages.drain(..) {
        debug.log_message(&message);

        debug.update_started();
        let command = future_runtime.enter(|| application.update(message));
        debug.update_finished();

        run_command(
            command,
            future_runtime,
            clipboard,
            proxy,
            window,
            graphics_info,
        );
    }

    // let subscription = application.subscription();
    // future_runtime.track(subscription);
}

/// Runs the actions of a [`Command`].
pub fn run_command<Message: 'static + std::fmt::Debug + Send, E: Executor>(
    command: Command<Message>,
    future_runtime: &mut FutureRuntime<E, Proxy<Message>, Message>,
    clipboard: &mut Clipboard,
    proxy: &mut winit::event_loop::EventLoopProxy<Message>,
    window: &winit::window::Window,
    _graphics_info: impl FnOnce() -> compositor::Information + Copy,
) {
    use emg_native::command;
    use emg_native::system;
    use emg_native::window;

    for action in command.actions() {
        match action {
            command::Action::Future(future) => {
                future_runtime.spawn(future);
            }
            command::Action::Clipboard(action) => match action {
                clipboard::Action::Read(tag) => {
                    let message = tag(clipboard.read());

                    proxy
                        .send_event(message)
                        .expect("Send message to event loop");
                }
                clipboard::Action::Write(contents) => {
                    clipboard.write(contents);
                }
            },
            command::Action::Window(action) => match action {
                window::Action::Resize { width, height } => {
                    window.set_inner_size(winit::dpi::LogicalSize { width, height });
                    //TODO make resize
                }
                window::Action::Move { x, y } => {
                    window.set_outer_position(winit::dpi::LogicalPosition { x, y });
                }
            },
            command::Action::System(action) => match action {
                system::Action::QueryInformation(_tag) => {
                    #[cfg(feature = "system")]
                    {
                        let graphics_info = _graphics_info();
                        let proxy = proxy.clone();

                        let _ = std::thread::spawn(move || {
                            let information = crate::system::information(graphics_info);

                            let message = _tag(information);

                            proxy
                                .send_event(message)
                                .expect("Send message to event loop")
                        });
                    }
                }
            },
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod platform {
    pub fn run<T, F>(
        mut event_loop: winit::event_loop::EventLoop<T>,
        event_handler: F,
    ) -> Result<(), crate::Error>
    where
        F: 'static
            + FnMut(
                winit::event::Event<'_, T>,
                &winit::event_loop::EventLoopWindowTarget<T>,
                &mut winit::event_loop::ControlFlow,
            ),
    {
        use winit::platform::run_return::EventLoopExtRunReturn;
        //TODO try not use run_return, use run
        event_loop.run_return(event_handler);

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
mod platform {
    pub fn run<T, F>(event_loop: winit::event_loop::EventLoop<T>, event_handler: F) -> !
    where
        F: 'static
            + FnMut(
                winit::event::Event<'_, T>,
                &winit::event_loop::EventLoopWindowTarget<T>,
                &mut winit::event_loop::ControlFlow,
            ),
    {
        event_loop.run(event_handler)
    }
}
