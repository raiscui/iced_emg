/*
 * @Author: Rais
 * @Date: 2021-03-04 12:16:31
 * @LastEditTime: 2021-05-21 09:02:32
 * @LastEditors: Rais
 * @Description:
 */
use std::{cell::RefCell, rc::Rc};

use emg_orders::Orders;
use iced::{Color, Element, Error, Settings};

use crate::{Application, Command, GTreeBuilderElement, GraphType, Subscription};

pub trait Sandbox {
    /// The type of __messages__ your [`Sandbox`] will produce.
    type Message: std::fmt::Debug + Send + Clone;

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
    fn update(&mut self, graph: &mut GraphType<Self::Message>, message: Self::Message);

    /// Returns the widgets to display in the [`Sandbox`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    fn view<'a>(&self, g: &'a GraphType<'_, Self::Message>) -> Element<'a, Self::Message>;

    /// Returns the background color of the [`Sandbox`].
    ///
    /// By default, it returns [`Color::WHITE`].
    fn background_color(&self) -> Color {
        Color::WHITE
    }

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

    fn tree_build<'a>(this: Rc<RefCell<Self>>) -> GTreeBuilderElement<'a, Self::Message>;

    /// Runs the [`Sandbox`].
    ///
    /// On native platforms, this method will take control of the current thread
    /// and __will NOT return__.
    ///
    /// It should probably be that last thing you call in your `main` function.
    /// # Errors
    ///
    /// error never returned at Web
    fn run(settings: Settings<()>) -> Result<(), Error>
    where
        Self: 'static + Sized,
    {
        #[allow(clippy::unit_arg)]
        <Self as Application>::run(settings)
    }
}
#[allow(clippy::use_self)]
impl<T> Application for T
where
    T: Sandbox,
{
    type Executor = crate::runtime::executor::Null;
    type Flags = ();
    type Message = T::Message;

    fn new(_flags: (), _orders: &impl Orders<Self::Message>) -> (Self, Command<T::Message>) {
        (T::new(), Command::none())
    }

    fn title(&self) -> String {
        T::title(self)
    }

    fn update(
        &mut self,
        graph: &mut GraphType<T::Message>,
        _orders: &impl Orders<T::Message>,
        message: T::Message,
    ) -> Command<T::Message> {
        T::update(self, graph, message);

        Command::none()
    }

    fn subscription(&self) -> Subscription<T::Message> {
        Subscription::none()
    }

    fn view<'a>(&self, g: &'a GraphType<'_, T::Message>) -> Element<'a, T::Message> {
        T::view(self, g)
    }
    fn tree_build<'a>(
        this: Rc<RefCell<Self>>,
        _orders: impl Orders<Self::Message> + 'static,
    ) -> GTreeBuilderElement<'a, T::Message> {
        T::tree_build(this)
    }
}
