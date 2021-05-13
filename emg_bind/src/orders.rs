use emg_animation::Msg as AmMsg;
use emg_animation::Tick;
/*
 * @Author: Rais
 * @Date: 2021-05-12 18:07:36
 * @LastEditTime: 2021-05-13 13:37:50
 * @LastEditors: Rais
 * @Description:
 */
use iced_web::Bus;
use tracing::{debug_span, error};
use wasm_bindgen::UnwrapThrowExt;
use web_sys::window;

use crate::map_callback_return_to_option_ms;
use emg_orders::Orders;
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
    time::Duration,
};

// ────────────────────────────────────────────────────────────────────────────────

// ────────────────────────────────────────────────────────────────────────────────
pub enum ShouldRender {
    Render,
    ForceRenderNow,
    Skip,
}

pub enum Effect<Message> {
    Msg(Option<Message>),
    // Notification(Notification),
    TriggeredHandler(Box<dyn FnOnce() -> Option<Message>>),
}

// impl<Message> From<Box<dyn FnOnce() -> Option<Message>>> for Effect<Message> {
//     fn from(v: Box<dyn FnOnce() -> Option<Message>>) -> Self {
//         Self::TriggeredHandler(v)
//     }
// }

// impl<Ms: 'static, OtherMs: 'static> MessageMapper<Ms, OtherMs> for Effect<Ms> {
//     type SelfWithOtherMs = Effect<OtherMs>;
//     fn map_msg(self, f: impl FnOnce(Ms) -> OtherMs + 'static + Clone) -> Effect<OtherMs> {
//         match self {
//             Effect::Msg(msg) => Effect::Msg(msg.map(f)),
//             Effect::Notification(notification) => Effect::Notification(notification),
//             Effect::TriggeredHandler(handler) => {
//                 Effect::TriggeredHandler(Box::new(move || handler().map(f)))
//             }
//         }
//     }
// }

// type StoredPopstate = RefCell<Option<Closure<dyn FnMut(web_sys::Event)>>>;

#[allow(clippy::type_complexity)]
pub(crate) struct OrdersData<Message: 'static, TickMsg> {
    // pub model: RefCell<Option<Mdl>>,
    // pub(crate) root_el: RefCell<Option<El<Ms>>>,
    // pub popstate_closure: StoredPopstate,
    // pub hashchange_closure: StoredPopstate,
    // pub window_event_handler_manager: RefCell<EventHandlerManager<Ms>>,
    // pub sub_manager: RefCell<SubManager<Ms>>,
    // pub msg_listeners: RefCell<Vec<Box<dyn Fn(&Ms)>>>,
    // pub scheduled_render_handle: RefCell<Option<util::RequestAnimationFrameHandle>>,
    pub after_next_render_callbacks: RefCell<Vec<Box<dyn FnOnce(TickMsg) -> Option<Message>>>>,
    pub render_info: Cell<Option<TickMsg>>,
}
// ────────────────────────────────────────────────────────────────────────────────

#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct OrdersContainer<Message>
where
    Message: 'static,
    // INodes: IntoNodes<Ms>,
{
    pub(crate) should_render: Rc<Cell<ShouldRender>>,
    pub(crate) data: Rc<OrdersData<Message, Tick>>,
    bus: Bus<Message>, // pub(crate) effects: VecDeque<Effect<Ms>>,
                       // app: App<Ms, Mdl, INodes>
}

impl<Message> OrdersContainer<Message>
where
    Message: 'static,
{
    pub fn new(bus: Bus<Message>) -> Self {
        Self {
            should_render: Rc::new(Cell::new(ShouldRender::Render)),
            // effects: VecDeque::<Effect<Ms>>::new(),
            // app,
            data: Rc::new(OrdersData {
                // model: RefCell::new(None),
                // root_el: RefCell::new(None),
                // popstate_closure: RefCell::new(None),
                // hashchange_closure: RefCell::new(None),
                // window_event_handler_manager: RefCell::new(EventHandlerManager::new()),
                // sub_manager: RefCell::new(SubManager::new()),
                // msg_listeners: RefCell::new(Vec::new()),
                // scheduled_render_handle: RefCell::new(None),
                after_next_render_callbacks: RefCell::new(Vec::new()),
                render_info: Cell::new(None),
            }),
            bus,
        }
    }
}

impl<Message> Orders<Message> for OrdersContainer<Message>
where
    Message: 'static,
{
    type AppMs = Message;
    type TickMsg = Tick;
    // type Mdl = Mdl;
    // type INodes = INodes;

    // ────────────────────────────────────────────────────────────────────────────────
    fn reset_render(&self) {
        self.should_render.set(ShouldRender::Render);
    }
    fn process_after_render_queue(&self, new_render_timestamp: f64) {
        if self.data.after_next_render_callbacks.borrow().is_empty() {
            return;
        }
        let build_time = debug_span!("build_time");
        let tick = build_time.in_scope(|| {
            // let new_render_timestamp = window()
            //     .expect("get window")
            //     .performance()
            //     .expect("get `Performance`")
            //     .now();

            Tick::new(new_render_timestamp)
        });
        // let mut queue: VecDeque<Effect<Message>> = self
        //     .data
        //     .after_next_render_callbacks
        //     .replace(Vec::new())
        //     .into_iter()
        //     // .scan(
        //     //     |tick, callback| Some(Effect::TriggeredHandler(Box::new(move || callback(*tick)))),
        //     // )
        //     .map(|callback| Effect::TriggeredHandler(Box::new(move || callback(tick))))
        //     .collect();

        self.data
            .after_next_render_callbacks
            .replace(Vec::new())
            .into_iter()
            //TODO:  for_each or just once?
            .for_each(|callback| {
                self.process_queue_message(callback(tick));
            })
    }
    // fn process_effect_queue(&self, queue: VecDeque<Effect<Message>>) {
    //     let process_queue_time = debug_span!("process_queue_time");
    //     {
    //         let _g = process_queue_time.enter();

    //         if std::thread::panicking() {
    //             error!("I don't know what is ");
    //             return;
    //         }
    //         //pop_front is get old one
    //         while let Some(effect) = queue.pop_front() {
    //             match effect {
    //                 Effect::Msg(_msg) => {
    //                     todo!();
    //                     // let mut new_effects = self.process_queue_message(msg);
    //                     // queue.append(&mut new_effects);
    //                 }
    //                 // Effect::Notification(notification) => {
    //                 // let mut new_effects = self.process_queue_notification(&notification);
    //                 // queue.append(&mut new_effects);
    //                 // }
    //                 Effect::TriggeredHandler(handler) => {
    //                     // let mut new_effects = self.process_queue_message(handler());
    //                     self.process_queue_message(handler());
    //                     // queue.append(&mut new_effects);
    //                 }
    //             }
    //         }
    //     }
    // }

    fn process_queue_message(&self, message: Option<Message>) {
        // TODO: 确定在这里以及 receiver 里面同时 set 是否 ok
        // self.should_render.set(ShouldRender::Render);

        if let Some(message) = message {
            // for l in self.data.msg_listeners.borrow().iter() {
            //     (l)(&message)
            // }

            // (self.cfg.update)(
            //     message,
            //     &mut self.data.model.borrow_mut().as_mut().unwrap(),
            //     &mut orders,
            // );
            //TODO:  send msg to update
            self.bus.publish(message);
        }

        // match self.should_render.get() {
        //     ShouldRender::Render => self.schedule_render(),
        //     ShouldRender::ForceRenderNow => {
        //         self.cancel_scheduled_render();
        //         self.rerender_vdom();
        //     }
        //     ShouldRender::Skip => (),
        // };
        // orders.effects
    }

    // ────────────────────────────────────────────────────────────────────────────────

    // #[allow(clippy::redundant_closure)]
    // fn proxy<ChildMs: 'static>(
    //     &mut self,
    //     f: impl FnOnce(ChildMs) -> Ms + 'static + Clone,
    // ) -> OrdersProxy<ChildMs, Ms, Mdl, INodes> {
    //     OrdersProxy::new(self, move |child_ms| f.clone()(child_ms))
    // }
    // ────────────────────────────────────────────────────────────────────────────────

    // fn render(&mut self) -> &mut Self {
    //     self.should_render = ShouldRender::Render;
    //     self
    // }

    // fn force_render_now(&mut self) -> &mut Self {
    //     self.should_render = ShouldRender::ForceRenderNow;
    //     self
    // }

    // fn skip(&mut self) -> &mut Self {
    //     self.should_render = ShouldRender::Skip;
    //     self
    // }

    // fn notify(&mut self, message: impl Any + Clone) -> &mut Self {
    //     self.effects
    //         .push_back(Effect::Notification(Notification::new(message)));
    //     self
    // }

    // fn send_msg(&mut self, msg: Ms) -> &mut Self {
    //     self.effects.push_back(Effect::Msg(Some(msg)));
    //     self
    // }

    // fn perform_cmd<MsU: 'static>(&mut self, cmd: impl Future<Output = MsU> + 'static) -> &mut Self {
    //     let app = self.app.clone();

    //     let handler = map_callback_return_to_option_ms!(
    //         dyn Fn(MsU) -> Option<Ms>,
    //         identity,
    //         "Cmds can return only Msg, Option<Msg> or ()!",
    //         Box
    //     );

    //     let cmd = cmd.map(move |msg| app.mailbox().send(handler(msg)));
    //     CmdManager::perform_cmd(cmd);
    //     self
    // }

    // fn perform_cmd_with_handle<MsU: 'static>(
    //     &mut self,
    //     cmd: impl Future<Output = MsU> + 'static,
    // ) -> CmdHandle {
    //     let app = self.app.clone();

    //     let handler = map_callback_return_to_option_ms!(
    //         dyn Fn(MsU) -> Option<Ms>,
    //         identity,
    //         "Cmds can return only Msg, Option<Msg> or ()!",
    //         Box
    //     );

    //     let cmd = cmd.map(move |msg| app.mailbox().send(handler(msg)));
    //     CmdManager::perform_cmd_with_handle(cmd)
    // }

    // fn clone_app(&self) -> App<Self::AppMs, Self::Mdl, Self::INodes> {
    //     self.app.clone()
    // }

    // fn msg_mapper(&self) -> Rc<dyn Fn(Ms) -> Self::AppMs> {
    //     Rc::new(identity)
    // }

    fn after_next_render<MsU: 'static>(
        &self,
        callback: impl FnOnce(Self::TickMsg) -> MsU + 'static,
    ) -> &Self {
        let callback = map_callback_return_to_option_ms!(
            dyn FnOnce(Self::TickMsg) -> Option<Message>,
            callback,
            "Callback can return only Msg, Option<Msg> or ()!",
            Box
        );

        self.data
            .after_next_render_callbacks
            .borrow_mut()
            .push(callback);
        self
    }

    // fn subscribe<MsU: 'static, SubMs: 'static + Clone>(
    //     &mut self,
    //     handler: impl FnOnce(SubMs) -> MsU + Clone + 'static,
    // ) -> &mut Self {
    //     let handler = map_callback_return_to_option_ms!(
    //         dyn Fn(SubMs) -> Option<Ms>,
    //         handler.clone(),
    //         "Handler can return only Msg, Option<Msg> or ()!",
    //         Rc
    //     );

    //     self.app
    //         .data
    //         .sub_manager
    //         .borrow_mut()
    //         .subscribe(move |sub_ms| handler(sub_ms));
    //     self
    // }

    // fn subscribe_with_handle<MsU: 'static, SubMs: 'static + Clone>(
    //     &mut self,
    //     handler: impl FnOnce(SubMs) -> MsU + Clone + 'static,
    // ) -> SubHandle {
    //     let handler = map_callback_return_to_option_ms!(
    //         dyn Fn(SubMs) -> Option<Ms>,
    //         handler.clone(),
    //         "Handler can return only Msg, Option<Msg> or ()!",
    //         Rc
    //     );

    //     self.app
    //         .data
    //         .sub_manager
    //         .borrow_mut()
    //         .subscribe_with_handle(move |sub_ms| handler(sub_ms))
    // }

    // fn stream<MsU: 'static>(&mut self, stream: impl Stream<Item = MsU> + 'static) -> &mut Self {
    //     let app = self.app.clone();

    //     let handler = map_callback_return_to_option_ms!(
    //         dyn Fn(MsU) -> Option<Ms>,
    //         identity,
    //         "Streams can stream only Msg, Option<Msg> or ()!",
    //         Box
    //     );

    //     let stream = stream.map(move |msg| app.mailbox().send(handler(msg)));
    //     StreamManager::stream(stream);
    //     self
    // }

    // fn stream_with_handle<MsU: 'static>(
    //     &mut self,
    //     stream: impl Stream<Item = MsU> + 'static,
    // ) -> StreamHandle {
    //     let app = self.app.clone();

    //     let handler = map_callback_return_to_option_ms!(
    //         dyn Fn(MsU) -> Option<Ms>,
    //         identity,
    //         "Streams can stream only Msg, Option<Msg> or ()!",
    //         Box
    //     );

    //     let stream = stream.map(move |msg| app.mailbox().send(handler(msg)));
    //     StreamManager::stream_with_handle(stream)
    // }
}
