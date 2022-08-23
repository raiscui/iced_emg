//! EMG application.
use emg_futures::FutureRuntime;
/*
 * @Author: Rais
 * @Date: 2021-03-04 10:02:43
 * @LastEditTime: 2022-08-23 15:56:01
 * @LastEditors: Rais
 * @Description:
 */
#[allow(unused)]
use emg_state::use_state_impl::{get_caller_location, get_caller_location2};
use seed_styles::GlobalStyleSV;
use tracing::{debug, debug_span, trace_span};

use crate::{
    command, futures,
    g_node::node_item_rc_sv::{GelType, GraphType},
    g_tree_builder::{GTreeBuilderElement, GTreeBuilderFn},
    orders::OrdersContainer,
    settings::Settings,
    Bus, Command, Executor,
};
use emg_orders::Orders;

use std::{cell::RefCell, fmt, rc::Rc};

use dodrio::{self, bumpalo};

/// An EMG edition interactive web application.
///
pub trait Application {
    /// The [`Executor`] that will run commands and subscriptions.
    type Executor: Executor;

    /// The type of __messages__ your [`Application`] will produce.
    type Message: Send + Clone + fmt::Debug + PartialEq; // TODO need move to T

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
    fn view(&self, g: &GraphType<Self::Message>) -> GelType<Self::Message>;

    /// Returns the event [`Subscription`] for the current state of the
    /// application.
    ///
    /// A [`Subscription`] will be kept alive as long as you keep returning it,
    /// and the __messages__ produced will be handled by
    /// [`update`](#tymethod.update).
    ///
    /// By default, this method returns an empty [`Subscription`].
    // fn subscription(&self) -> Subscription<Self::Message> {
    //     Subscription::none()
    // }

    /// Build dom `GTreeBuilderElement`, will callonce at `run`
    fn tree_build(
        this: Rc<RefCell<Self>>,
        orders: impl Orders<Self::Message> + 'static,
    ) -> GTreeBuilderElement<Self::Message>;

    /// Runs the [`Application`].
    /// # Errors
    /// never error,  `iced::Error`
    fn run(flags: Settings<Self::Flags>) -> crate::Result
    where
        Self: 'static + Sized,
    {
        use futures::stream::StreamExt;

        let _g = trace_span!("application::run").entered();

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();

        let (sender, receiver) = futures::channel::mpsc::unbounded();

        let mut future_runtime = FutureRuntime::new(
            Self::Executor::new().expect("Create executor"),
            sender.clone(),
        );
        let orders = OrdersContainer::<Self::Message>::new(Bus::new(sender.clone()));

        let (app, command) = future_runtime.enter(|| Self::new(flags.flags, &orders));

        let mut title = app.title();
        document.set_title(&title);

        run_command(command, &mut future_runtime);
        // runtime.spawn(command);

        let application = Rc::new(RefCell::new(app));

        // ─────────────────────────────────────────────────────────────────

        let emg_graph = GraphType::<Self::Message>::default();
        let root = Self::tree_build(Rc::clone(&application), orders.clone());
        // emg_graph.handle_root_in_topo(&root);
        let emg_graph_rc_refcell = Rc::new(RefCell::new(emg_graph));
        emg_graph_rc_refcell.handle_root_in_topo(&root);

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
            // let (command, subscription) = runtime.enter(|| {
            let command = future_runtime.enter(|| {
                let update_span = trace_span!("application->update");
                // let sub_span = trace_span!("application->subscription");
                let command = update_span.in_scope(|| {
                    application.borrow_mut().update(
                        &mut emg_graph_rc_refcell.borrow_mut(),
                        &orders,
                        message,
                    )
                });
                // let subscription = sub_span.in_scope(|| application.borrow().subscription());

                // (command, subscription)
                command
            });

            let new_title = application.borrow().title();

            {
                let _g = trace_span!("application->spawn command").entered();
                // runtime.spawn(command);
                run_command(command, &mut future_runtime);
            }
            {
                let _g = trace_span!("application->track subscription").entered();
                // runtime.track(subscription);
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

fn run_command<Message: 'static + Send, E: Executor>(
    command: Command<Message>,
    runtime: &mut FutureRuntime<
        E,
        emg_futures::futures::channel::mpsc::UnboundedSender<Message>,
        Message,
    >,
) {
    for action in command.actions() {
        match action {
            command::Action::Future(future) => {
                runtime.spawn(future);
            }
        }
    }
}

struct Instance<A: Application>
where
    A::Message: 'static,
{
    application: Rc<RefCell<A>>,
    bus: Bus<A::Message>,
    g: Rc<RefCell<GraphType<A::Message>>>,
}

trait CssNode<'a> {
    fn node(self, bump: &'a bumpalo::Bump) -> dodrio::Node<'a>;
}
impl<'a> CssNode<'a> for GlobalStyleSV {
    fn node(self, bump: &'a bumpalo::Bump) -> dodrio::Node<'a> {
        use dodrio::builder::{style, text};

        let mut declarations = bumpalo::collections::Vec::new_in(bump);
        declarations.push(text("html { height: 100% }"));
        declarations.push(text(
            "body { height: 100%; margin: 0; padding: 0; font-family: sans-serif }",
        ));
        declarations.push(text("* { margin: 0; padding: 0 }"));
        declarations.push(text(
            "button { border: none; cursor: pointer; outline: none }",
        ));
        declarations.push(text(
            bumpalo::collections::String::from_str_in(self.render().as_str(), bump).into_bump_str(),
        ));

        // for declaration in self.rules.values() {
        //     declarations.push(text(*declaration));
        // }

        style(bump).children(declarations).finish()
    }
}

// fn get_caller_location3() {
//     get_caller_location2();
// }
impl<'a, A> dodrio::Render<'a> for Instance<A>
where
    A: Application,
{
    fn render(&self, context: &mut dodrio::RenderContext<'a>) -> dodrio::Node<'a> {
        use dodrio::builder::div;
        let _g = debug_span!("application->render").entered();
        debug!("render");

        let app_ref = self.application.borrow();
        let emg_graph_ref = self.g.borrow();

        let view_span = trace_span!("application->view");
        let element = view_span.in_scope(|| app_ref.view(&*emg_graph_ref));

        // get_caller_location2();
        // get_caller_location3();
        // let mut css = Css::new();
        let css = GlobalStyleSV::default_topo();

        let node_span = trace_span!("application->element.node");
        //TODO 集成 node() 动作到 anchor 增量计算. (可能要 缓存 每一个节点生成的 Node)
        let node = node_span.in_scope(|| {
            element
                .as_dyn_node_widget()
                .node(context.bump, &self.bus, &css)
        });

        trace_span!("application-> dodrio .finish").in_scope(|| {
            div(context.bump)
                .attr("style", "width: 100%; height: 100%")
                .children(vec![css.node(context.bump), node])
                .finish()
        })
    }
}
