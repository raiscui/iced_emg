/*
 * @Author: Rais
 * @Date: 2022-08-11 14:11:24
 * @LastEditTime: 2023-03-01 18:21:10
 * @LastEditors: Rais
 * @Description:
 */
//! Build interactive cross-platform applications.

use emg::EdgeIndex;
use emg_common::{IdStr, Pos, Vector};
use emg_element::{
    graph_edit::{GraphEdit, GraphEditManyMethod},
    GTreeBuilderFn, GraphMethods,
};
use emg_orders::Orders;
use emg_state::{state_lit::StateVarLit, StateAnchor};
use tracing::instrument;

use crate::{element, window, Command, Executor, Settings};
use std::{cell::RefCell, rc::Rc};

// pub use emg_native::application::StyleSheet;

pub trait Application: Sized {
    /// The [`Executor`] that will run commands and subscriptions.
    ///
    /// The [default executor] can be a good starting point!
    ///
    /// [`Executor`]: Self::Executor
    /// [default executor]: crate::executor::Default
    type Executor: Executor;

    /// The type of __messages__ your [`Application`] will produce.
    type Message: std::fmt::Debug + Send;

    // /// The theme of your [`Application`].
    // type Theme: Default + StyleSheet;

    /// The data needed to initialize your [`Application`].
    type Flags;

    type GraphType: GraphMethods<Self::Message> + Default + Clone;
    type GraphEditor: GraphEdit + GraphEditManyMethod;
    type Orders: Orders<Self::Message>;

    /// Initializes the [`Application`] with the flags provided to
    /// [`run`] as part of the [`Settings`].
    ///
    /// Here is where you should return the initial state of your app.
    ///
    /// Additionally, you can return a [`Command`] if you need to perform some
    /// async action in the background on startup. This is useful if you want to
    /// load state from a file, perform an initial HTTP request, etc.
    ///
    /// [`run`]: Self::run
    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>);

    /// Returns the current title of the [`Application`].
    ///
    /// This title can be dynamic! The runtime will automatically update the
    /// title of your application when necessary.
    fn title(&self) -> String;

    /// Handles a __message__ and updates the state of the [`Application`].
    ///
    /// This is where you define your __update logic__. All the __messages__,
    /// produced by either user interactions or commands, will be handled by
    /// this method.
    ///
    /// Any [`Command`] returned will be executed immediately in the background.
    fn update(
        &mut self,
        graph: Self::GraphEditor,
        orders: &Self::Orders,
        message: Self::Message,
    ) -> Command<Self::Message>;

    /// Returns the widgets to display in the [`Application`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    // fn view(&self, g: &element::GraphType<Self::Message>) -> element::GelType<Self::Message>;
    // fn view(&mut self) -> GElement<Self::Message>;

    fn root_eix(&self) -> EdgeIndex;

    // fn ctx(
    //     &self,
    //     g: &element::GraphType<Self::Message>,
    // ) -> StateAnchor<crate::runtime::PaintCtx<crate::renderer::SceneCtx>>;

    // /// Returns the current [`Theme`] of the [`Application`].
    // ///
    // /// [`Theme`]: Self::Theme
    // fn theme(&self) -> Self::Theme {
    //     Self::Theme::default()
    // }

    // /// Returns the current [`Style`] of the [`Theme`].
    // ///
    // /// [`Style`]: <Self::Theme as StyleSheet>::Style
    // /// [`Theme`]: Self::Theme
    // fn style(&self) -> <Self::Theme as StyleSheet>::Style {
    //     <Self::Theme as StyleSheet>::Style::default()
    // }

    // /// Returns the event [`Subscription`] for the current state of the
    // /// application.
    // ///
    // /// A [`Subscription`] will be kept alive as long as you keep returning it,
    // /// and the __messages__ produced will be handled by
    // /// [`update`](#tymethod.update).
    // ///
    // /// By default, this method returns an empty [`Subscription`].
    // fn subscription(&self) -> Subscription<Self::Message> {
    //     Subscription::none()
    // }

    /// Returns the current [`Application`] mode.
    ///
    /// The runtime will automatically transition your application if a new mode
    /// is returned.
    ///
    /// Currently, the mode only has an effect in native platforms.
    ///
    /// By default, an application will run in windowed mode.
    fn mode(&self) -> window::Mode {
        window::Mode::Windowed
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

    fn tree_build(&self, orders: Self::Orders) -> element::GTreeBuilderElement<Self::Message>;

    /// Runs the [`Application`].
    ///
    /// On native platforms, this method will take control of the current thread
    /// until the [`Application`] exits.
    ///
    /// On the web platform, this method __will NOT return__ unless there is an
    /// [`Error`] during startup.
    ///
    /// # Errors
    ///
    /// Error: [`crate::Error`]
    fn run(settings: Settings<Self::Flags>) -> crate::Result
    where
        Self: 'static,
        Instance<Self>: crate::runtime::Application<
            Flags = Self::Flags,
            Message = Self::Message,
            Orders = crate::runtime::OrdersContainer<Self::Message>,
        >,

        crate::renderer::window::Compositor: crate::runtime::Compositor<
            Renderer = <Instance<Self> as crate::runtime::GraphProgram>::Renderer,
        >,
    {
        #[allow(clippy::needless_update)]
        let renderer_settings = crate::renderer::Settings {
            // default_font: settings.default_font,
            // default_text_size: settings.default_text_size,
            // text_multithreading: settings.text_multithreading,
            // antialiasing: if settings.antialiasing {
            //     Some(crate::renderer::settings::Antialiasing::MSAAx4)
            // } else {
            //     None
            // },
            width: settings.window.size.0 as usize,
            height: settings.window.size.1 as usize,

            ..crate::renderer::Settings::from_env()
        };

        Ok(crate::runtime::application::run::<
            Instance<Self>,
            Self::Executor,
            crate::renderer::window::Compositor,
        >(settings.into(), renderer_settings)?)
    }
}

pub struct Instance<A: Application>(A, StateVarLit<Option<EdgeIndex>>);

impl<A> crate::runtime::Program for Instance<A>
where
    A: Application,
{
    type Message = A::Message;
    type GraphType = A::GraphType;
    type Orders = A::Orders;
    type GraphEditor = A::GraphEditor;

    fn update(
        &mut self,
        graph: Self::GraphEditor,
        orders: &Self::Orders,
        message: Self::Message,
    ) -> Command<Self::Message>
    where
        Self::GraphEditor: GraphEdit + GraphEditManyMethod,
    {
        self.0.update(graph, orders, message)
    }
}
impl<A> crate::runtime::GraphProgram for Instance<A>
where
    // ─────────────────────────────────────────────────────────────────────────────
    A: Application,

    <A as Application>::GraphType: GraphMethods<
        <A as Application>::Message,
        SceneCtx = <crate::Renderer as crate::runtime::renderer::Renderer>::SceneCtx,
    >,

    // ─────────────────────────────────────────────────────────────────────
    <A as Application>::Message: 'static,
    // ─────────────────────────────────────────────────────────────────────
    <A as Application>::GraphType: GTreeBuilderFn<
        <A as Application>::Message,
        GraphType = <A as Application>::GraphType,
        GraphEditor = <A as Application>::GraphEditor,
    >,
    // ─────────────────────────────────────────────────────────────────────
{
    type Renderer = crate::Renderer;

    type GTreeWithBuilder = <A as Application>::GraphType;

    fn graph_setup(
        &self,
        renderer: &Self::Renderer,
        orders: Self::Orders,
    ) -> Self::GTreeWithBuilder {
        let emg_graph = <Self::GraphType>::default();
        let tree = self.0.tree_build(orders);
        let emg_graph_rc_refcell: Self::GTreeWithBuilder = emg_graph;
        emg_graph_rc_refcell.handle_root_in_topo(&tree);
        emg_graph_rc_refcell
    }

    fn root_eix(&self) -> EdgeIndex {
        self.0.root_eix()
    }

    // #[instrument(skip(self, g))]
    // fn view(&self, g: &Self::GraphType) -> Self::RefedGelType {
    //     self.0.view(g)
    // }

    //build_runtime_sas

    #[instrument(skip(self, g, events, cursor_position))]
    fn build_ctx(
        &self,
        g: &Self::GraphType,
        painter: &StateAnchor<crate::runtime::PaintCtx>,
        events: &StateAnchor<Vector<crate::runtime::EventWithFlagType>>,
        cursor_position: &StateAnchor<Option<Pos>>,
    ) -> crate::runtime::EventAndCtx<Self::Message, Self::Renderer> {
        self.1.set(Some(self.root_eix()));

        g.runtime_prepare(self.1.watch(), painter, events, cursor_position)
    }
}

impl<A> crate::runtime::Application for Instance<A>
where
    A: Application,
    <A as Application>::Message: 'static,

    <A as Application>::GraphType: GraphMethods<
        <A as Application>::Message,
        SceneCtx = <crate::Renderer as crate::runtime::renderer::Renderer>::SceneCtx,
    >,

    <A as Application>::GraphType: GTreeBuilderFn<
        <A as Application>::Message,
        GraphType = <A as Application>::GraphType,
        GraphEditor = <A as Application>::GraphEditor,
    >,
{
    type Flags = A::Flags;

    fn new(flags: Self::Flags) -> (Self, Command<<A as Application>::Message>) {
        let (app, command) = A::new(flags);
        (Self(app, StateVarLit::new(None)), command)
    }

    fn title(&self) -> String {
        self.0.title()
    }

    // fn theme(&self) -> A::Theme {
    //     self.0.theme()
    // }

    // fn style(&self) -> <A::Theme as StyleSheet>::Style {
    //     self.0.style()
    // }

    fn mode(&self) -> emg_winit::Mode {
        match self.0.mode() {
            window::Mode::Windowed => emg_winit::Mode::Windowed,
            window::Mode::Fullscreen => emg_winit::Mode::Fullscreen,
            window::Mode::Hidden => emg_winit::Mode::Hidden,
        }
    }

    // fn subscription(&self) -> Subscription<Self::Message> {
    //     self.0.subscription()
    // }

    fn scale_factor(&self) -> f64 {
        self.0.scale_factor()
    }

    fn should_exit(&self) -> bool {
        self.0.should_exit()
    }
}
