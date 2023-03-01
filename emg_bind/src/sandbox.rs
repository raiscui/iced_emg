use emg::EdgeIndex;
use emg_element::{
    graph_edit::{GraphEdit, GraphEditManyMethod, GraphEditor},
    GraphMethods,
};
use emg_orders::Orders;

use crate::{application::Instance, element, Application, Command, Settings};

pub trait Sandbox: std::default::Default {
    /// The type of __messages__ your [`Sandbox`] will produce.
    type Message: std::fmt::Debug + Send + 'static;
    type GraphType = element::GraphType<Self::Message>;
    // type GraphType: GraphMethods<Self::Message> + Default;
    type Orders = crate::runtime::OrdersContainer<Self::Message>;
    type GraphEditor: GraphEdit + GraphEditManyMethod = GraphEditor<Self::Message>;

    // type Orders: Orders<Self::Message>;

    /// Initializes the [`Sandbox`].
    ///
    /// Here is where you should return the initial state of your app.
    #[must_use]
    fn new() -> Self {
        Self::default()
    }

    /// Returns the current title of the [`Sandbox`].
    ///
    /// This title can be dynamic! The runtime will automatically update the
    /// title of your application when necessary.
    fn title(&self) -> String {
        String::from("emg gui")
    }

    /// Handles a __message__ and updates the state of the [`Sandbox`].
    ///
    /// This is where you define your __update logic__. All the __messages__,
    /// produced by user interactions, will be handled by this method.
    fn update(&mut self, graph: Self::GraphEditor, orders: &Self::Orders, message: Self::Message);

    /// Returns the widgets to display in the [`Sandbox`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    // fn view(&self, g: &element::GraphType<Self::Message>) -> element::GelType<Self::Message>;
    fn root_eix(&self) -> EdgeIndex;

    // fn ctx(
    //     &self,
    //     g: &element::GraphType<Self::Message>,
    // ) -> StateAnchor<crate::runtime::PaintCtx<crate::renderer::SceneCtx>>;

    // /// Returns the current [`Theme`] of the [`Sandbox`].
    // ///
    // /// If you want to use your own custom theme type, you will have to use an
    // /// [`Application`].
    // ///
    // /// By default, it returns [`Theme::default`].
    // fn theme(&self) -> Theme {
    //     Theme::default()
    // }

    // /// Returns the current style variant of [`theme::Application`].
    // ///
    // /// By default, it returns [`theme::Application::default`].
    // fn style(&self) -> theme::Application {
    //     theme::Application::default()
    // }

    /// Returns the scale factor of the [`Sandbox`].
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

    /// Returns whether the [`Sandbox`] should be terminated.
    ///
    /// By default, it returns `false`.
    fn should_exit(&self) -> bool {
        false
    }
    fn tree_build(&self, orders: Self::Orders) -> element::GTreeBuilderElement<Self::Message>;

    /// Runs the [`Sandbox`].
    ///
    /// On native platforms, this method will take control of the current thread
    /// and __will NOT return__.
    ///
    /// It should probably be that last thing you call in your `main` function.
    /// [`Error`] during startup.
    ///
    /// # Errors
    ///
    /// Error: [`crate::Error`]
    fn run(settings: Settings<()>) -> crate::Result
    where
        Self: Application<Flags = ()> + 'static,
        Instance<Self>: crate::runtime::Application<
            Flags = (),
            Message = <Self as Application>::Message,
            Orders = crate::runtime::OrdersContainer<<Self as Application>::Message>,
        >,
        crate::renderer::window::Compositor: crate::runtime::Compositor<
            Renderer = <Instance<Self> as crate::runtime::GraphProgram>::Renderer,
        >,
    {
        <Self as Application>::run(settings)
    }
}

impl<SB> Application for SB
where
    SB: Sandbox,
    <SB as Sandbox>::Message: 'static,
    <SB as Sandbox>::GraphType: GraphMethods<<SB as Sandbox>::Message> + Default,
    <SB as Sandbox>::Orders: Orders<<SB as Sandbox>::Message>,
    <SB as Sandbox>::GraphEditor: GraphEdit + GraphEditManyMethod, // <SB as Sandbox>::RcRefCellGraphType: GraphEdit + GraphEditManyMethod,
{
    //TODO use cargo.toml choose emg_futures backend
    // type Executor = emg_futures::backend::null::Executor;
    type Executor = emg_futures::backend::default::Executor;
    type Flags = ();
    type Message = SB::Message;
    // type Orders = crate::runtime::OrdersContainer<Self::Message>;
    // type GraphType = element::GraphType<Self::Message>;

    type GraphType = SB::GraphType;
    type Orders = SB::Orders;
    type GraphEditor = SB::GraphEditor;

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (SB::new(), Command::none())
    }

    fn title(&self) -> String {
        SB::title(self)
    }

    fn update(
        &mut self,
        graph: Self::GraphEditor,
        orders: &Self::Orders,
        message: Self::Message,
    ) -> Command<Self::Message> {
        SB::update(self, graph, orders, message);

        Command::none()
    }
    // fn view(&self, g: &element::GraphType<Self::Message>) -> element::GelType<Self::Message> {
    //     T::view(self, g)
    // }
    fn root_eix(&self) -> EdgeIndex {
        SB::root_eix(self)
    }

    // fn ctx(
    //     &self,
    //     g: &element::GraphType<Self::Message>,
    // ) -> StateAnchor<crate::runtime::PaintCtx<crate::renderer::SceneCtx>> {
    //     T::ctx(self, g)
    // }

    // fn theme(&self) -> Self::Theme {
    //     T::theme(self)
    // }

    // fn style(&self) -> theme::Application {
    //     T::style(self)
    // }

    // fn subscription(&self) -> Subscription<T::Message> {
    //     Subscription::none()
    // }

    fn scale_factor(&self) -> f64 {
        SB::scale_factor(self)
    }

    fn should_exit(&self) -> bool {
        SB::should_exit(self)
    }
    fn tree_build(&self, orders: Self::Orders) -> element::GTreeBuilderElement<Self::Message> {
        SB::tree_build(self, orders)
    }
}
