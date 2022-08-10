/*
 * @Author: Rais
 * @Date: 2021-03-04 12:16:31
 * @LastEditTime: 2022-08-10 15:15:25
 * @LastEditors: Rais
 * @Description:
 */
use std::{cell::RefCell, rc::Rc};

use emg_orders::Orders;

use crate::{
    g_node::node_item_rc_sv::{GelType, GraphType},
    g_tree_builder::GTreeBuilderElement,
    settings::Settings,
    Application, Command,
};

pub trait Sandbox {
    /// The type of __messages__ your [`Sandbox`] will produce.
    type Message: std::fmt::Debug + Send + Clone + PartialEq;

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
    fn view(&self, g: &GraphType<Self::Message>) -> GelType<Self::Message>;

    fn tree_build(this: Rc<RefCell<Self>>) -> GTreeBuilderElement<Self::Message>;

    /// Runs the [`Sandbox`].
    ///
    /// On native platforms, this method will take control of the current thread
    /// and __will NOT return__.
    ///
    /// It should probably be that last thing you call in your `main` function.
    /// # Errors
    ///
    /// error never returned at Web
    fn run(settings: Settings<()>) -> crate::Result
    where
        Self: 'static + Sized,
    {
        #[allow(clippy::unit_arg)]
        <Self as Application>::run(settings)
    }
}
#[allow(clippy::use_self)]
impl<SBox> Application for SBox
where
    SBox: Sandbox,
{
    type Executor = emg_futures::backend::null::Executor;
    type Flags = ();
    type Message = SBox::Message;

    fn new(_flags: (), _orders: &impl Orders<Self::Message>) -> (Self, Command<SBox::Message>) {
        (SBox::new(), Command::none())
    }

    fn title(&self) -> String {
        SBox::title(self)
    }

    fn update(
        &mut self,
        graph: &mut GraphType<SBox::Message>,
        _orders: &impl Orders<SBox::Message>,
        message: SBox::Message,
    ) -> Command<SBox::Message> {
        SBox::update(self, graph, message);

        Command::none()
    }

    // fn subscription(&self) -> Subscription<T::Message> {
    //     Subscription::none()
    // }

    fn view(&self, g: &GraphType<SBox::Message>) -> GelType<SBox::Message> {
        SBox::view(self, g)
    }
    fn tree_build(
        this: Rc<RefCell<Self>>,
        _orders: impl Orders<Self::Message>,
    ) -> GTreeBuilderElement<SBox::Message> {
        SBox::tree_build(this)
    }
}
