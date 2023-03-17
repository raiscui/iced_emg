/*
 * @Author: Rais
 * @Date: 2022-08-13 13:11:58
 * @LastEditTime: 2023-03-17 10:51:35
 * @LastEditors: Rais
 * @Description:
 */
//! Create interactive, native cross-platform applications.
mod state;

use std::{hash::BuildHasherDefault, rc::Rc};

use emg_common::{
    im::{hashmap::HashMapPool, vector::RRBPool, HashMap},
    Vector,
};
use emg_hasher::CustomHasher;
use emg_orders::Orders;
pub use state::State;
use winit::event_loop::EventLoopBuilder;

use crate::conversion::{self, ev::EventState};
use crate::mouse;
use crate::{
    clipboard::{self, Clipboard},
    orders::OrdersContainer,
};
use crate::{Command, Debug, Executor, FutureRuntime, Mode, Proxy, Settings};
use emg_element::{GTreeBuilderFn, GraphProgram};
use emg_futures::futures;
use emg_futures::futures::channel::mpsc;
use emg_graphics_backend::window::{
    compositor::{self, CompositorSetting},
    Compositor,
};
use emg_native::{
    event::{EventIdentify, EventWithFlagType},
    renderer::Renderer,
    Bus, Program, EVENT_DEBOUNCE,
};
use emg_state::state_lit::StateVarLit;
use emg_state::CloneStateAnchor;
use tracing::{debug, debug_span, info, info_span, instrument, warn};

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
    mut compositor_settings: C::Settings,
) -> Result<(), crate::Error>
where
    A: Application<Orders = OrdersContainer<<A as Program>::Message>> + 'static,
    // <A as Program>::Message: 'static,
    // <A as Program>::GraphType: GraphMethods<<A as Program>::Message>,
    // Rc<RefCell<<A as Program>::GraphType>>:
    // GTreeBuilderFn<<A as Program>::Message, GraphType = <A as Program>::GraphType>,
    E: Executor + 'static,
    C: Compositor<Renderer = A::Renderer> + 'static,
{
    use futures::task;
    use futures::Future;

    let mut debug = Debug::new();
    debug.startup_started();

    // let event_loop = EventLoop::with_user_event();
    let event_loop = EventLoopBuilder::with_user_event().build();
    let user_event_proxy = event_loop.create_proxy();

    let future_runtime = {
        let proxy = Proxy::new(event_loop.create_proxy());
        let executor = E::new().map_err(crate::Error::ExecutorCreationFailed)?;

        FutureRuntime::new(executor, proxy)
    };
    // ─────────────────────────────────────────────────────────────────────────────
    // bus

    let bus_sender = user_event_proxy.clone();

    let orders: A::Orders = OrdersContainer::new(Bus::new(move |msg| {
        bus_sender
            .send_event(msg)
            .expect("OrdersContainer Send user message");
    }));
    // ─────────────────────────────────────────────────────────────────────────────
    // app

    let (application, init_command) = {
        let flags = settings.flags;

        // let (app, command) = future_runtime.enter(|| Self::new(flags.flags, &orders));
        future_runtime.enter(|| A::new(flags))
    };
    // ─────────────────────────────────────────────────────────────────────────────

    let builder = settings.window.into_builder(
        &application.title(),
        application.mode(),
        event_loop.primary_monitor(),
        application.scale_factor(),
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

    // let size: (f64, f64) = window.inner_size().to_logical::<f64>(dpr).into();
    // info!("Window size: {:?} {:?}", size, dpr);
    // emg_layout::global_width().set(size.0);
    // emg_layout::global_height().set(size.1);
    // ────────────────────────────────────────────────────────────────────────────────
    compositor_settings.set_vp_scale_factor(application.scale_factor() * window.scale_factor());
    let (compositor, renderer) = C::new(compositor_settings, &window)?;

    // future_runtime.track(subscription);

    let (mut sender, receiver) = mpsc::unbounded();

    // let emg_graph = A::GraphType::default();
    // let emg_graph_rc_refcell = Rc::new(RefCell::new(emg_graph));
    // emg_graph_rc_refcell.handle_root_in_topo(&root);

    let orders2 = orders.clone();
    let emg_graph_rc_refcell = application.graph_setup(&renderer, orders2);

    let mut instance = Box::pin(run_instance::<A, E, C>(
        application,
        compositor,
        renderer,
        future_runtime,
        user_event_proxy,
        debug,
        receiver,
        init_command,
        window,
        settings.exit_on_close_request,
        emg_graph_rc_refcell,
        orders,
    ));

    let mut context = task::Context::from_waker(task::noop_waker_ref());

    platform::run(event_loop, move |event, _, control_flow| {
        use winit::event_loop::ControlFlow;

        if let &mut ControlFlow::ExitWithCode(_) = control_flow {
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

// #[instrument(skip_all)]
async fn run_instance<A, E, C>(
    mut application: A,
    mut compositor: C,
    mut renderer: A::Renderer,
    mut future_runtime: FutureRuntime<E, Proxy<A::Message>, A::Message>,
    mut proxy: winit::event_loop::EventLoopProxy<A::Message>,
    mut debug: Debug,
    mut receiver: mpsc::UnboundedReceiver<winit::event::Event<'_, A::Message>>,
    init_command: Command<A::Message>,
    window: winit::window::Window,
    exit_on_close_request: bool,
    g: A::GTreeWithBuilder,
    orders: A::Orders,
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

    let mut clipboard = Clipboard::connect(&window);

    let mut surface = compositor.create_surface(&window);
    let mut state = State::new(&application, &window);
    let mut viewport_version = state.viewport_version();
    let mut event_state = EventState::default();

    // let physical_size = state.physical_size();

    // compositor.configure_surface(&mut surface, physical_size.width, physical_size.height);

    // let mut user_interface = ManuallyDrop::new(build_user_interface(
    //     &mut application,
    //     user_interface::Cache::default(),
    //     &mut renderer,
    //     state.logical_size(),
    //     &mut debug,
    // ));

    let painter = state
        .vp_scale_factor_sa()
        .map(|sf| crate::PaintCtx::new(*sf));

    //view
    // let native_events: StateVar<Vector<EventWithFlagType>> = use_state(||Vector::new());
    let event_vec_pool = RRBPool::new(8);
    let event_hm_pool = HashMapPool::new(8);

    let native_events: StateVarLit<Vector<EventWithFlagType>> =
        StateVarLit::new(Vector::with_pool(&event_vec_pool));
    //
    let mut latest_event_state: HashMap<
        EventIdentify,
        emg_native::Event,
        BuildHasherDefault<CustomHasher>,
    > = HashMap::with_pool_hasher(
        &event_hm_pool,
        BuildHasherDefault::<CustomHasher>::default(),
    );

    //
    let event_debouncer =
        native_events
            .watch()
            .map_mut(Vector::with_pool(&event_vec_pool), move |out, ev_list| {
                let mut changed = false;
                if !out.is_empty() {
                    out.clear();
                    changed = true;
                }
                let iter = ev_list.iter();
                for (evf, ev) in iter {
                    if  EVENT_DEBOUNCE.intersects(evf) {

                        latest_event_state
                        .entry(*evf)
                        .and_modify(|latest_ev| {
                        debug!(target:"winit_event","has old state,\nevf:{:?}\nold:\n{:?}\nnew:\n{:?}",evf,latest_ev,ev);

                            if latest_ev != ev {
                                *latest_ev = ev.clone();
                                out.push_back((*evf, ev.clone()));
                                changed = true;
                            }
                            else{
                                warn!(target:"winit_event","same,ignored");
                            }
                        })
                        .or_insert_with(|| {
                            out.push_back((*evf, ev.clone()));
                            changed = true;
                            ev.clone()
                        });

                    }else{
                        debug!(target:"winit_event",?evf,?ev);

                        out.push_back((*evf, ev.clone()));
                            changed = true;
                    }


                    // let latest_ev = latest_event_state.get_mut(evf);
                    // if latest_ev.is_none() {
                    //     latest_event_state.insert(*evf, ev.clone());
                    //     out.push_back((*evf, ev.clone()));
                    //     changed = true;
                    // } else {
                    //     let latest_ev = latest_ev.unwrap();
                    //     if latest_ev != ev {
                    //         latest_event_state.insert(*evf, ev.clone());
                    //         out.push_back((*evf, ev.clone()));
                    //         changed = true;
                    //     }
                    // }
                }
                info!(target:"winit_event",?changed);

                changed
            });

    // let native_events_is_empty = native_events.watch().map(|v| v.is_empty());
    let native_events_is_empty = event_debouncer.map(|v| v.is_empty());

    let (event_matchs_sa, ctx_sa) = application.build_ctx(
        g.graph(),
        &painter,
        // &native_events.watch(),
        &event_debouncer,
        state.cursor_position(),
    );
    let mut ctx = ctx_sa.get();
    // let mut element = application.view(&g.graph());

    let mouse_interaction = mouse::Interaction::default();
    let mut messages = Vec::new();

    debug.startup_finished();

    run_command(
        init_command,
        &mut future_runtime,
        &mut clipboard,
        &mut proxy,
        &window,
        || compositor.fetch_information(),
    );

    while let Some(winit_event) = receiver.next().await {
        match winit_event {
            event::Event::MainEventsCleared => {
                let _span = info_span!(target:"winit_event","MainEventsCleared").entered();

                if !native_events_is_empty.get() {
                    info!(target:"winit_event",?native_events);

                    let event_matchs = event_matchs_sa.get();
                    //清空 native_events, 因为 event_matchs 已经获得, native_events使用完毕;
                    native_events.set(Default::default());

                    if !event_matchs.is_empty() {
                        for (_ei, ev, en_list) in event_matchs.iter().flatten().flatten() {
                            for en in en_list {
                                if let Some(msg) = en.call(ev) {
                                    messages.push(msg);
                                }
                            }
                        }
                    }
                }
                {
                    use nu_ansi_term::Color::Red;
                    use nu_ansi_term::Style;
                    info!(target:"winit_event","{}",Style::new().on(Red).paint(  "============= event processed end ============================="));
                }
                //NOTE  has events or messages now -------------------

                // for event in events.drain(..).zip(statuses.into_iter()) {
                //     future_runtime.broadcast(event);
                // }

                if !messages.is_empty()
                //TODO 实现 Outdated check, SceneFrag 变更 -> Outdated
                // || matches!(interface_state, user_interface::State::Outdated,)
                {
                    // let cache = ManuallyDrop::into_inner(user_interface).into_cache();

                    // TODO Update application
                    update(
                        &mut application,
                        &mut future_runtime,
                        &mut clipboard,
                        &mut proxy,
                        &mut debug,
                        &mut messages,
                        &window,
                        &g,
                        &orders,
                        || compositor.fetch_information(),
                    );

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

                let new_ctx = ctx_sa.get();
                //new_ctx == ctx
                if Rc::ptr_eq(&new_ctx, &ctx) {
                    //NOTE 不渲染,提前跳过
                    continue;
                } else {
                    ctx = new_ctx;
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
                info!(target:"winit_event","element painting");
                // element.paint(&mut ctx);
                // ctx = ctx_sa.get();

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
                // let _span = info_span!(target:"winit_event","UserEvent").entered();
                info!(target:"winit_event","UserEvent:{:?}",message);

                messages.push(message);
            }
            event::Event::RedrawRequested(_) => {
                // let _span = info_span!(target:"winit_event","RedrawRequested").entered();
                info!(target:"winit_event","RedrawRequested");

                // if physical_size.x == 0 || physical_size.y == 0 {
                //     continue;
                // }

                debug.render_started();
                let current_viewport_version = state.viewport_version();

                if viewport_version != current_viewport_version {
                    // let physical_size = state.physical_size();
                    let user_size = state.user_size();
                    // .try_cast::<u32>()
                    // .ok_or("user_size f64 cast to u32 cast error")
                    // .unwrap();

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
                    debug_span!(
                        "window_size",
                        vp_scale_factor = state.vp_scale_factor(),
                        "will configure_surface use user_size"
                    )
                    .in_scope(|| {});

                    compositor.configure_surface(
                        &mut surface,
                        user_size.x.round() as u32,
                        user_size.y.round() as u32,
                    );

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
                let _span = info_span!(target:"winit_event","LoopDestroyed").entered();

                renderer.on_loop_destroyed();
            }

            event::Event::WindowEvent {
                event: window_event,
                ..
            } => {
                // let _span = info_span!(target:"winit_event","WindowEvent").entered();
                info!(target:"winit_event",?window_event);
                // info!(target:"winit_event","window.scale_factor():{}",window.scale_factor());//2

                if requests_exit(&window_event, state.modifiers()) && exit_on_close_request {
                    break;
                }

                state.update(&window, &window_event, &mut debug);

                if let Some(event_with_flag) = conversion::window_event(
                    &window_event,
                    state.vp_scale_factor(),
                    state.modifiers(),
                    &mut event_state,
                ) {
                    // native_events.push(event);
                    native_events.update(
                        |ev| ev.extend(event_with_flag.iter().cloned()), // ev.push_back(event_with_flag)
                    );
                }

                if viewport_version != state.viewport_version() {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    // Manually drop the user interface
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
    graph: &A::GTreeWithBuilder,
    orders: &A::Orders,
    graphics_info: impl FnOnce() -> compositor::Information + Copy,
) {
    for message in messages.drain(..) {
        debug.log_message(&message);

        debug.update_started();
        let command = future_runtime.enter(|| application.update(graph.editor(), orders, message));
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
            //TODO check work
            command::Action::Future(future) => {
                future_runtime.spawn(future);
            }
            //TODO check work
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
            //TODO check work
            command::Action::Window(action) => match action {
                window::Action::Resize { width, height } => {
                    window.set_inner_size(winit::dpi::LogicalSize { width, height });
                    //TODO make resize
                }
                window::Action::Move { x, y } => {
                    window.set_outer_position(winit::dpi::LogicalPosition { x, y });
                }
            },
            //TODO check work
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
        let _ = event_loop.run_return(event_handler);

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
