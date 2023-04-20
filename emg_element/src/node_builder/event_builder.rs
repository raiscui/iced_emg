/*
 * @Author: Rais
 * @Date: 2022-09-05 20:56:05
 * @LastEditTime: 2023-04-09 21:12:37
 * @LastEditors: Rais
 * @Description:
 */
use std::{
    cell::{Cell, RefCell},
    hash::BuildHasherDefault,
    rc::Rc,
};

use derive_more::From;
use emg_hasher::CustomHasher;
use emg_native::{
    drag,
    event::{MultiLevelIdentify, MultiLevelIdentifyWithSwitch},
    EVENT_LONG_STATE_INIT, GLOBAL_PENETRATE_EVENTS,
};
use indexmap::IndexSet;
use tracing::{debug, debug_span, info};

use crate::platform::{event::EventIdentify, Event};
use emg_common::{
    im::{HashMap, HashSet, OrdSet},
    SmallVec, Vector,
};
use emg_state::{state_lit::StateVarLit, Dict};

// ─────────────────────────────────────────────────────────────────────────────

//TODO in web, EventCallbackFn has 3 arg, native need same arg
//TODO use TopoKey for PartialEq
type EventCallbackFn<Message> = Rc<dyn Fn(&Event) -> Option<Message>>;
type EventMessageFn<Message> = Rc<dyn Fn() -> Option<Message>>;

/// 3 arg event callback
pub struct EventCallback<Message>(EventIdentify, EventCallbackFn<Message>);
impl<Message> EventCallback<Message> {
    #[must_use]
    pub fn new<T: IntoOptionMs<Message>>(
        name: EventIdentify,
        cb: impl Fn(&Event) -> T + 'static,
        // cb: impl FnOnce() -> T + Clone + 'static,
    ) -> Self {
        Self(name, Rc::new(move |ev| cb(ev).into_option()))
    }
}
impl<Message> std::fmt::Debug for EventCallback<Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("EventCallback")
            .field(&self.0)
            .field(&"EventCallbackFn<Message>")
            .finish()
    }
}

impl<Message> Clone for EventCallback<Message> {
    fn clone(&self) -> Self {
        Self(self.0, self.1.clone())
    }
}

impl<Message> PartialEq for EventCallback<Message>
// where
//     Message: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.0 == other.0 {
            debug_assert_eq!(
                &*self.1 as *const _ as *const u8,
                &*other.1 as *const _ as *const u8
            );

            debug_assert!(std::ptr::eq(
                &*self.1 as *const _ as *const u8,
                &*other.1 as *const _ as *const u8
            ));
            true
        } else {
            debug_assert_ne!(
                &*self.1 as *const _ as *const u8,
                &*other.1 as *const _ as *const u8
            );
            debug_assert!(!std::ptr::eq(
                &*self.1 as *const _ as *const u8,
                &*other.1 as *const _ as *const u8
            ));
            false
        }
    }
}

pub struct EventMessage<Message>(EventIdentify, EventMessageFn<Message>);
impl<Message: 'static> EventMessage<Message> {
    #[must_use]
    pub fn new<T: IntoOptionMs<Message>>(
        name: EventIdentify,
        cb: impl Fn() -> T + 'static,
        // cb: impl FnOnce() -> T + Clone + 'static,
    ) -> Self {
        Self(name, Rc::new(move || cb().into_option()))
    }
}
impl<Message> std::fmt::Debug for EventMessage<Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("EventMessage")
            .field(&self.0)
            .field(&"EventMessageFn<Message>")
            .finish()
    }
}

impl<Message> Clone for EventMessage<Message> {
    fn clone(&self) -> Self {
        Self(self.0, self.1.clone())
    }
}

impl<Message> PartialEq for EventMessage<Message>
// where
//     Message: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        // self.0 == other.0

        //NOTE refpool
        //FIXME comparing trait object pointers compares a non-unique vtable address
        //comparing trait object pointers compares a non-unique vtable address
        // consider extracting and comparing data pointers only
        // for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#vtable_address_comparisons
        //
        // && Rc::ptr_eq(&self.1, &other.1)

        if self.0 == other.0 {
            debug_assert_eq!(
                &*self.1 as *const _ as *const u8,
                &*other.1 as *const _ as *const u8
            );

            debug_assert!(std::ptr::eq(
                &*self.1 as *const _ as *const u8,
                &*other.1 as *const _ as *const u8
            ));
            true
        } else {
            debug_assert_ne!(
                &*self.1 as *const _ as *const u8,
                &*other.1 as *const _ as *const u8
            );
            debug_assert!(!std::ptr::eq(
                &*self.1 as *const _ as *const u8,
                &*other.1 as *const _ as *const u8
            ));
            false
        }
    }
}
// ────────────────────────────────────────────────────────────────────────────────

auto trait MsgMarker {}
impl !MsgMarker for () {}
impl<Message> !MsgMarker for Option<Message> {}

pub trait IntoOptionMs<Message> {
    fn into_option(self) -> Option<Message>;
}

impl<Message: MsgMarker> IntoOptionMs<Message> for Message {
    fn into_option(self) -> Option<Message> {
        Some(self)
    }
}

impl<Message> IntoOptionMs<Message> for () {
    fn into_option(self) -> Option<Message> {
        Option::<Message>::None
    }
}

impl<Message> IntoOptionMs<Message> for Option<Message> {
    fn into_option(self) -> Option<Message> {
        self
    }
}
// ────────────────────────────────────────────────────────────────────────────────

#[derive(From)]
pub enum EventNode<Message> {
    // no arg
    Cb(EventCallback<Message>),
    // 3 arg
    CbMessage(EventMessage<Message>),
}

impl<Message> EventNode<Message> {
    #[inline]
    pub fn get_identify(&self) -> EventIdentify {
        match self {
            EventNode::Cb(x) => x.0,
            EventNode::CbMessage(x) => x.0,
        }
    }
    pub fn call(&self, event: &Event) -> Option<Message> {
        match self {
            EventNode::Cb(x) => {
                info!("EventNode::Cb call");
                //TODO real 3 arg
                (x.1)(event)
            }
            EventNode::CbMessage(x) => (x.1)(),
        }
    }
}

impl<Message> std::fmt::Debug for EventNode<Message>
// where
//     Message: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventNode::Cb(x) => f.debug_tuple("EventNode<Message>").field(x).finish(),
            EventNode::CbMessage(x) => f.debug_tuple("EventNode<Message>").field(x).finish(),
        }
    }
}

impl<Message> PartialEq for EventNode<Message> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Cb(l0), Self::Cb(r0)) => l0 == r0,
            (Self::CbMessage(l0), Self::CbMessage(r0)) => l0 == r0,
            _ => false,
        }
    }
}
impl<Message> Eq for EventNode<Message> where Message: PartialEq {}

impl<Message> Clone for EventNode<Message> {
    fn clone(&self) -> Self {
        match self {
            Self::Cb(arg0) => Self::Cb(arg0.clone()),
            Self::CbMessage(arg0) => Self::CbMessage(arg0.clone()),
        }
    }
}
#[derive(Debug, Clone, Default)]
pub(crate) struct EventLongTimeState {
    // pub drag: bool,
    pub penetrate: MultiLevelIdentifyWithSwitch,
    pub long_state: MultiLevelIdentifyWithSwitch,
}
pub struct EventListener<Message> {
    pub(crate) event_callbacks: Dict<EventIdentify, Vector<EventNode<Message>>>,
    pub(crate) event_long_state: Rc<RefCell<EventLongTimeState>>,
}

impl<Message> std::fmt::Debug for EventListener<Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBuilder")
            .field("event_callbacks", &self.event_callbacks)
            .finish()
    }
}

impl<Message> Clone for EventListener<Message> {
    fn clone(&self) -> Self {
        Self {
            event_callbacks: self.event_callbacks.clone(),
            event_long_state: self.event_long_state.clone(),
        }
    }
}

impl<Message> PartialEq for EventListener<Message> {
    fn eq(&self, other: &Self) -> bool {
        self.event_callbacks == other.event_callbacks
    }
}

impl<Message> EventListener<Message> {
    pub fn new() -> Self {
        //@-- init
        let ls = EventLongTimeState {
            penetrate: (*GLOBAL_PENETRATE_EVENTS.read()).clone(),
            long_state: (*EVENT_LONG_STATE_INIT.read()).clone(),
        };

        Self {
            event_callbacks: Dict::default(),
            event_long_state: Rc::new(RefCell::new(ls)),
        }
    }

    pub fn event_callbacks(&self) -> &Dict<EventIdentify, Vector<EventNode<Message>>> {
        &self.event_callbacks
    }
}

impl<Message: 'static> EventListener<Message> {
    pub(crate) fn register_listener(
        &mut self,
        event_name: EventIdentify,
        event_node: EventNode<Message>,
    ) {
        let _span = debug_span!("event", action = "register_listener").entered();
        self.event_prepare(event_name);
        self.register_listener_no_prepare(event_name, event_node);
    }
    fn event_prepare(&mut self, event_name: EventIdentify) {
        if event_name.lv1() == crate::platform::event::EventFlag::DND.bits() {
            // ─────────────────────────────────────────────────────

            let event_state = self.event_long_state.clone();
            let ei = crate::platform::drag::DRAG_START.into();
            let drag_on: EventNode<Message> = EventMessage::new(ei, move || {
                {
                    let mut es = event_state.borrow_mut();

                    es.long_state.insert(drag::DRAG.into(), true);

                    es.penetrate.insert(drag::DRAG.into(), true);
                    es.penetrate.insert(drag::DRAG_END.into(), true);
                }
                {
                    let mut w = GLOBAL_PENETRATE_EVENTS.write();
                    w.insert(drag::DRAG.into(), true);
                    w.insert(drag::DRAG_END.into(), true);
                }
            })
            .into();

            self.register_listener_no_prepare(ei, drag_on);

            // ─────────────────────────────────────────────────────────────────────────────

            let event_state = self.event_long_state.clone();
            let ei2 = crate::platform::drag::DRAG_END.into();
            let drag_off: EventNode<Message> = EventMessage::new(ei, move || {
                {
                    let mut es = event_state.borrow_mut();

                    es.long_state.insert(drag::DRAG.into(), false);

                    es.penetrate.insert(drag::DRAG.into(), false);
                    es.penetrate.insert(drag::DRAG_END.into(), false);
                }

                {
                    let mut w = GLOBAL_PENETRATE_EVENTS.write();
                    w.insert(drag::DRAG.into(), false);
                    w.insert(drag::DRAG_END.into(), false);
                }
            })
            .into();

            self.register_listener_no_prepare(ei2, drag_off);

            // ─────────────────────────────────────────────────────
        }
    }

    fn register_listener_no_prepare(
        &mut self,
        event_name: EventIdentify,
        event_node: EventNode<Message>,
    ) {
        let _span = debug_span!("event", action = "register_listener_no_prepare").entered();
        let entry = self.event_callbacks.entry(event_name);
        let v = entry.or_default();
        v.push_back(event_node);
        debug!("event list: {:#?}", v);
    }
}
