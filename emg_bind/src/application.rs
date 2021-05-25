//! EMG application.
/*
 * @Author: Rais
 * @Date: 2021-03-04 10:02:43
 * @LastEditTime: 2021-05-21 16:22:18
 * @LastEditors: Rais
 * @Description:
 */

use tracing::{debug, debug_span, trace, trace_span};

use crate::{orders::OrdersContainer, GTreeBuilderElement, GTreeBuilderFn, GraphType};
use emg_orders::Orders;

use std::{cell::RefCell, fmt, rc::Rc};

pub use crate::runtime::{futures, Command};

use crate::runtime::dodrio;
use crate::runtime::Bus;
use crate::runtime::Css;
pub use crate::runtime::Element;
use crate::Subscription;

#[doc(no_inline)]
use crate::runtime::Executor;

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
    fn new(
        flags: Self::Flags,
        orders: &impl Orders<Self::Message>,
    ) -> (Self, Command<Self::Message>)
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
    fn update(
        &mut self,
        graph: &mut GraphType<Self::Message>,
        orders: &impl Orders<Self::Message>,
        message: Self::Message,
    ) -> Command<Self::Message>;

    /// Returns the widgets to display in the [`Application`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    fn view<'a>(&self, g: &'a GraphType<'_, Self::Message>) -> Element<'a, Self::Message>;

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

    fn tree_build<'a>(
        this: Rc<RefCell<Self>>,
        orders: impl Orders<Self::Message> + 'static,
    ) -> GTreeBuilderElement<'a, Self::Message>;

    /// Runs the [`Application`].
    /// # Errors
    /// never error,  `iced::Error`
    fn run(flags: iced::Settings<Self::Flags>) -> iced::Result
    where
        Self: 'static + Sized,
    {
        use futures::stream::StreamExt;

        let _g = trace_span!("application::run").entered();

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();

        let (sender, receiver) = futures::channel::mpsc::unbounded();

        let mut runtime = iced_futures::Runtime::new(
            Self::Executor::new().expect("Create executor"),
            sender.clone(),
        );
        let mut orders = OrdersContainer::<Self::Message>::new(Bus::new(sender.clone()));

        let (app, command) = runtime.enter(|| Self::new(flags.flags, &orders));

        let mut title = app.title();
        document.set_title(&title);

        runtime.spawn(command);

        let application = Rc::new(RefCell::new(app));

        // ─────────────────────────────────────────────────────────────────

        let mut emg_graph = GraphType::<Self::Message>::default();
        let root = Self::tree_build(Rc::clone(&application), orders.clone());
        emg_graph.handle_root_in_topo(&root);
        let emg_graph_rc_refcell = Rc::new(RefCell::new(emg_graph));
        // let emg_graph_rc = (emg_graph);
        // GraphType::<Self::Message>::init();
        // GraphType::<Self::Message>::get_mut_graph_with(|g| {
        //     crate::handle_root(g, root);
        // });
        // ────────────────────────────────────────────────────────────────────────────────

        let instance = Instance {
            application: application.clone(),
            bus: Bus::new(sender),
            g: Rc::clone(&emg_graph_rc_refcell),
        };

        let vdom = dodrio::Vdom::new(&body, instance);
        *orders.vdom.borrow_mut() = Some(vdom.weak());
        // ─────────────────────────────────────────────────────────────────
        let event_loop = receiver.for_each(move |message| {
            //TODO check render enum;
            orders.reset_render();
            let _g_event_loop = debug_span!("event_loop", ?message).entered();
            debug!("receiver-message: {:?}", message);
            let (command, subscription) = runtime.enter(|| {
                let update_span = trace_span!("application->update");
                let sub_span = trace_span!("application->subscription");
                let command = update_span.in_scope(|| {
                    application.borrow_mut().update(
                        &mut emg_graph_rc_refcell.borrow_mut(),
                        &orders,
                        message,
                    )
                });
                let subscription = sub_span.in_scope(|| application.borrow().subscription());

                (command, subscription)
            });

            let new_title = application.borrow().title();

            {
                let _g = trace_span!("application->spawn command").entered();
                runtime.spawn(command);
            }
            {
                let _g = trace_span!("application->track subscription").entered();
                runtime.track(subscription);
            }

            if title != new_title {
                document.set_title(&new_title);
                //TODO: uncomment this
                title = new_title;
            }
            {
                let _g = debug_span!("application->schedule_render_with_orders").entered();
                debug!("schedule_render_with_orders");
                vdom.weak().schedule_render_with_orders(orders.clone());
            }
            // {
            //     let _g = trace_span!("application->track subscription").entered();
            //     runtime.track(subscription);
            // }

            futures::future::ready(())
        });

        wasm_bindgen_futures::spawn_local(event_loop);
        // ─────────────────────────────────────────────────────────────────

        Ok(())
    }
}

struct Instance<'a, A: Application> {
    application: Rc<RefCell<A>>,
    bus: Bus<A::Message>,
    g: Rc<RefCell<GraphType<'a, A::Message>>>,
}

impl<'a, A> dodrio::Render<'a> for Instance<'_, A>
where
    A: Application,
{
    fn render(&self, context: &mut dodrio::RenderContext<'a>) -> dodrio::Node<'a> {
        use dodrio::builder::div;
        let _g = debug_span!("application->render").entered();
        debug!("render");

        let ui = self.application.borrow();
        let emg_graph_ref = self.g.borrow();

        let view_span = trace_span!("application->view");
        let element = view_span.in_scope(|| ui.view(&*emg_graph_ref));

        let mut css = Css::new();

        let node_span = trace_span!("application->element.node");
        let node = node_span.in_scope(|| element.node(context.bump, &self.bus, &mut css));

        trace_span!("application-> dodrio .finish").in_scope(|| {
            div(context.bump)
                .attr("style", "width: 100%; height: 100%")
                .children(vec![css.node(context.bump), node])
                .finish()
        })
    }
}
