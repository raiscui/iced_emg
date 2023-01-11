use emg_state::StateAnchor;

use crate::{element, Application, Command, Settings};

pub trait Sandbox {
    /// The type of __messages__ your [`Sandbox`] will produce.
    type Message: std::fmt::Debug + Send;

    /// Initializes the [`Sandbox`].
    ///
    /// Here is where you should return the initial state of your app.
    fn new() -> Self;

    /// Returns the current title of the [`Sandbox`].
    ///
    /// This title can be dynamic! The runtime will automatically update the
    /// title of your application when necessary.
    fn title(&self) -> String;

    /// Handles a __message__ and updates the state of the [`Sandbox`].
    ///
    /// This is where you define your __update logic__. All the __messages__,
    /// produced by user interactions, will be handled by this method.
    fn update(&mut self, message: Self::Message);

    /// Returns the widgets to display in the [`Sandbox`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    // fn view(&self, g: &element::GraphType<Self::Message>) -> element::GelType<Self::Message>;
    fn root_id(&self) -> &str;

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
    fn tree_build(
        &self,
        // orders: impl Orders<Self::Message> + 'static,
    ) -> element::GTreeBuilderElement<Self::Message>;

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
        Self: 'static + Sized,
    {
        <Self as Application>::run(settings)
    }
}

impl<T> Application for T
where
    T: Sandbox,
{
    //TODO use cargo.toml choose emg_futures backend
    // type Executor = emg_futures::backend::null::Executor;
    type Executor = emg_futures::backend::default::Executor;
    type Flags = ();
    type Message = T::Message;

    fn new(_flags: ()) -> (Self, Command<T::Message>) {
        (T::new(), Command::none())
    }

    fn title(&self) -> String {
        T::title(self)
    }

    fn update(&mut self, message: T::Message) -> Command<T::Message> {
        T::update(self, message);

        Command::none()
    }
    // fn view(&self, g: &element::GraphType<Self::Message>) -> element::GelType<Self::Message> {
    //     T::view(self, g)
    // }
    fn root_id(&self) -> &str {
        T::root_id(self)
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
        T::scale_factor(self)
    }

    fn should_exit(&self) -> bool {
        T::should_exit(self)
    }
    fn tree_build(
        &self,
        // orders: impl Orders<Self::Message> + 'static,
    ) -> element::GTreeBuilderElement<T::Message> {
        T::tree_build(self)
    }
}
