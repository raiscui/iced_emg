//! EMG application.
/*
 * @Author: Rais
 * @Date: 2021-03-04 10:02:43
 * @LastEditTime: 2021-03-04 15:52:13
 * @LastEditors: Rais
 * @Description:
 */

use crate::{GTreeBuilderElement, GraphType};

use std::{borrow::Cow, cell::RefCell, fmt, rc::Rc};

pub use iced_web::{futures, Command};

use iced_web::dodrio;
use iced_web::Bus;
use iced_web::Css;
pub use iced_web::Element;
pub use iced_web::Subscription;

#[doc(no_inline)]
use iced_web::Executor;

/// An EMG edition interactive web application.
///
pub trait Application {
    /// The [`Executor`] that will run commands and subscriptions.
    type Executor: Executor;

    /// The type of __messages__ your [`Application`] will produce.
    type Message: Send + Clone + fmt::Debug;

    /// The data needed to initialize your [`Application`].
    type Flags;

    /// Initializes the [`Application`].
    ///
    /// Here is where you should return the initial state of your app.
    ///
    /// Additionally, you can return a [`Command`] if you need to perform some
    /// async action in the background on startup. This is useful if you want to
    /// load state from a file, perform an initial HTTP request, etc.
    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>)
    where
        Self: Sized;

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
    fn update(&mut self, message: Self::Message) -> Command<Self::Message>;

    /// Returns the widgets to display in the [`Application`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    fn view(&mut self) -> Element<'_, Self::Message>;

    /// Returns the event [`Subscription`] for the current state of the
    /// application.
    ///
    /// A [`Subscription`] will be kept alive as long as you keep returning it,
    /// and the __messages__ produced will be handled by
    /// [`update`](#tymethod.update).
    ///
    /// By default, this method returns an empty [`Subscription`].
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    fn tree_build(&self) -> GTreeBuilderElement<'_, Self::Message>;

    /// Runs the [`Application`].
    fn run(flags: Self::Flags)
    where
        Self: 'static + Sized,
    {
        use futures::stream::StreamExt;

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();

        let (sender, receiver) = futures::channel::mpsc::unbounded();

        let mut runtime = iced_futures::Runtime::new(
            Self::Executor::new().expect("Create executor"),
            sender.clone(),
        );

        let (app, command) = runtime.enter(|| Self::new(flags));

        let mut title = app.title();
        document.set_title(&title);

        runtime.spawn(command);

        let application = Rc::new(RefCell::new(app));

        use crate::graph_store::GraphStore;

        let app_for_treebuild = application.clone();
        let ref_app_for_treebuild = app_for_treebuild.borrow();

        let root = ref_app_for_treebuild.tree_build();
        let mut g = GraphType::<'_, Self::Message>::default();
        crate::handle_root(&mut g, root);
        let rcg = Rc::new(RefCell::new(g));
        // let cowg = Cow::<'static, GraphType<'_, Self::Message>>::Owned(g);
        // GraphType::<Self::Message>::init();
        // GraphType::<Self::Message>::get_mut_graph_with(|g| {
        //     crate::handle_root(g, root);
        // });

        let instance = Instance {
            application: application.clone(),
            bus: Bus::new(sender),
            g: Rc::clone(&rcg),
        };

        let vdom = dodrio::Vdom::new(&body, instance);

        let event_loop = receiver.for_each(move |message| {
            let (command, subscription) = runtime.enter(|| {
                let command = application.borrow_mut().update(message);
                let subscription = application.borrow().subscription();

                (command, subscription)
            });

            let new_title = application.borrow().title();

            runtime.spawn(command);
            runtime.track(subscription);

            if title != new_title {
                document.set_title(&new_title);

                title = new_title;
            }

            vdom.weak().schedule_render();

            futures::future::ready(())
        });

        wasm_bindgen_futures::spawn_local(event_loop);
    }
}

struct Instance<'a, A: Application> {
    application: Rc<RefCell<A>>,
    bus: Bus<A::Message>,
    g: Rc<RefCell<GraphType<'a, A::Message>>>,
}

impl<'a, 'b, A> dodrio::Render<'a> for Instance<'b, A>
where
    A: Application,
{
    fn render(&self, context: &mut dodrio::RenderContext<'a>) -> dodrio::Node<'a> {
        use dodrio::builder::*;

        let mut ui = self.application.borrow_mut();
        let element = ui.view();
        let mut css = Css::new();

        let node = element.node(context.bump, &self.bus, &mut css);

        div(context.bump)
            .attr("style", "width: 100%; height: 100%")
            .children(vec![css.node(context.bump), node])
            .finish()
    }
}
