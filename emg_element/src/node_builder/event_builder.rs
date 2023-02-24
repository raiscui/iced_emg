/*
 * @Author: Rais
 * @Date: 2022-09-05 20:56:05
 * @LastEditTime: 2023-02-24 16:40:49
 * @LastEditors: Rais
 * @Description:
 */
use std::rc::Rc;

use derive_more::From;
use tracing::{debug, debug_span, info};

use emg_common::{mouse, Vector};
use emg_native::event::EventFlag;
use emg_state::Dict;

// ─────────────────────────────────────────────────────────────────────────────

//TODO in web, EventCallbackFn has 3 arg, native need same arg
//TODO use TopoKey for PartialEq
type EventCallbackFn<Message> = Rc<dyn Fn(&mut i32) -> Option<Message>>;
type EventMessageFn<Message> = Rc<dyn Fn() -> Option<Message>>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventIdentify(u32, u32);

impl EventIdentify {
    #[inline]
    pub const fn contains(&self, other: Self) -> bool {
        self.0 == other.0 && (self.1 & other.1) == other.1
    }
}

impl From<mouse::EventFlag> for EventIdentify {
    fn from(x: mouse::EventFlag) -> Self {
        Self(emg_native::event::MOUSE.bits(), x.bits())
    }
}

impl From<(EventFlag, u32)> for EventIdentify {
    fn from(x: (EventFlag, u32)) -> Self {
        Self(x.0.bits(), x.1)
    }
}

/// 3 arg event callback
pub struct EventCallback<Message>(EventIdentify, EventCallbackFn<Message>);
impl<Message> EventCallback<Message> {
    #[must_use]
    pub fn new(name: EventIdentify, cb: EventCallbackFn<Message>) -> Self {
        Self(name, cb)
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
    pub fn get_identify(&self) -> EventIdentify {
        match self {
            EventNode::Cb(x) => x.0,
            EventNode::CbMessage(x) => x.0,
        }
    }
    pub fn call(&self) -> Option<Message> {
        match self {
            EventNode::Cb(x) => {
                info!("EventNode::Cb call");
                //TODO real 3 arg
                (x.1)(&mut 1)
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

pub struct EventListener<Message> {
    pub(crate) event_callbacks: Dict<EventIdentify, Vector<EventNode<Message>>>,
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
        Self {
            event_callbacks: Dict::new(),
        }
    }

    pub fn event_callbacks(&self) -> &Dict<EventIdentify, Vector<EventNode<Message>>> {
        &self.event_callbacks
    }
}

impl<Message> EventListener<Message> {
    pub(crate) fn register_listener(
        &mut self,
        event_name: EventIdentify,
        event_node: EventNode<Message>,
    ) {
        let _span = debug_span!("event", action = "register_listener").entered();
        let entry = self.event_callbacks.entry(event_name);
        let v = entry.or_insert_with(Vector::new);
        v.push_back(event_node);
        debug!("event list: {:#?}", v);
    }
    // fn register_event(
    //     mut self,
    //     event_name: EventNameString,
    //     event_node: EventNode<Message>,
    // ) -> Self {
    //     self.event_callbacks = self.event_callbacks.update_with(
    //         event_name,
    //         vector![event_node],
    //         |mut old_v, new_v| {
    //             old_v.append(new_v);
    //             old_v
    //         },
    //     );
    //     self
    // }
}
