/*
 * @Author: Rais
 * @Date: 2021-03-08 18:20:22
 * @LastEditTime: 2022-06-15 16:33:24
 * @LastEditors: Rais
 * @Description:
 */

#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::ptr_eq)]
use dyn_partial_eq::DynPartialEq;
// use dyn_clone::DynClone;
// ────────────────────────────────────────────────────────────────────────────────

use derive_more::From;

use emg_core::IdStr;
use seed_styles::GlobalStyleSV;
use tracing::{debug, trace};

use crate::{
    dodrio::{self, bumpalo, Node, RootRender, VdomWeak},
    map_fn_callback_return_to_option_ms, Bus, GElement, Widget,
};
use std::{collections::VecDeque, rc::Rc, string::String};
// use emg_core::Vector;
// ────────────────────────────────────────────────────────────────────────────────
// #[dyn_partial_eq]
// pub trait NodeBuilder<Message>: DynPartialEq // DynPartialEq
// // Message: 'static,
// {
//     fn generate_element_builder<'b>(
//         &self,
//         bump: &'b bumpalo::Bump,
//         bus: &Bus<Message>,
//         style_sheet: &GlobalStyleSV,
//     ) -> ElementBuilder<
//         'b,
//         bumpalo::collections::Vec<'b, Listener<'b>>,
//         bumpalo::collections::Vec<'b, Attribute<'b>>,
//         bumpalo::collections::Vec<'b, Node<'b>>,
//     >;
// }

// impl<Message> core::cmp::Eq for dyn NodeBuilder<Message> + '_ {}

// impl<Message> core::cmp::PartialEq for dyn NodeBuilder<Message> + '_ {
//     fn eq(&self, other: &Self) -> bool {
//         self.box_eq(other.as_any())
//     }
// }
// impl<Message> core::cmp::PartialEq<dyn NodeBuilder<Message> + '_>
//     for Box<dyn NodeBuilder<Message> + '_>
// {
//     fn eq(&self, other: &dyn NodeBuilder<Message>) -> bool {
//         self.box_eq(other.as_any())
//     }
// }

// default impl<Message, Who> Widget<Message> for Who
// where
//     Message: 'static + Clone + std::cmp::PartialEq,
//     Who: NodeBuilder<Message>,
// {
//     fn node<'b>(
//         &self,
//         bump: &'b bumpalo::Bump,
//         bus: &Bus<Message>,
//         style_sheet: &GlobalStyleSV,
//     ) -> dodrio::Node<'b> {
//         self.generate_element_builder(bump, bus, style_sheet)
//             .finish()
//     }
// }

// impl<Message> core::cmp::PartialEq<dyn NodeBuilder<Message> + '_>
//     for Rc<dyn NodeBuilder<Message> + '_>
// {
//     fn eq(&self, other: &dyn NodeBuilder<Message>) -> bool {
//         self.box_eq(other.as_any())
//     }
// }

// ────────────────────────────────────────────────────────────────────────────────
// impl<Message> core::cmp::PartialEq for Box<dyn NodeBuilder<Message> + '_> {
//     fn eq(&self, other: &Self) -> bool {
//         self.box_eq(other.as_any())
//     }
// }

// impl<Message> core::cmp::PartialEq<&Self> for Box<dyn NodeBuilder<Message> + '_> {
//     fn eq(&self, other: &&Self) -> bool {
//         self.box_eq(other.as_any())
//     }
// }
// ────────────────────────────────────────────────────────────────────────────────

// pub type ListenerCallback = Box<dyn EventCallbackClone + 'static>;

// pub trait EventCallbackClone: Fn(&mut dyn RootRender, VdomWeak, web_sys::Event) + DynClone {}
// dyn_clone::clone_trait_object!(EventCallbackClone);

// ────────────────────────────────────────────────────────────────────────────────

// pub trait EventCbClone<Message>:
//     Fn(&mut dyn RootRender, VdomWeak, web_sys::Event) -> Option<Message>
// {
//     fn clone_box(&self) -> Box<dyn EventCbClone<Message>>;
// }

// impl<Message, T> EventCbClone<Message> for T
// where
//     T: 'static + Fn(&mut dyn RootRender, VdomWeak, web_sys::Event) -> Option<Message> + Clone,
// {
//     fn clone_box(&self) -> Box<dyn EventCbClone<Message>> {
//         Box::new(self.clone())
//     }
// }

// impl<Message> Clone for Box<dyn EventCbClone<Message>> {
//     fn clone(&self) -> Self {
//         (**self).clone_box()
//     }
// }
// ────────────────────────────────────────────────────────────────────────────────
// pub trait EventMessageCbClone<Message>: Fn() -> Message {
//     fn clone_box(&self) -> Box<dyn EventMessageCbClone<Message>>;
// }

// impl<Message, T> EventMessageCbClone<Message> for T
// where
//     T: 'static + Fn() -> Message + Clone,
// {
//     fn clone_box(&self) -> Box<dyn EventMessageCbClone<Message>> {
//         Box::new(self.clone())
//     }
// }

// impl<Message> Clone for Box<dyn EventMessageCbClone<Message>> {
//     fn clone(&self) -> Self {
//         (**self).clone_box()
//     }
// }
// ────────────────────────────────────────────────────────────────────────────────

// pub struct EventCallbackCloneStatic<T>(T)
// where
//     T: EventCallbackClone + 'static;

// impl<T> EventCallbackCloneStatic<T>
// where
//     T: EventCallbackClone + 'static,
// {
//     pub fn new(f: T) -> Self {
//         Self(f)
//     }
// }

type EventNameString = IdStr;

#[derive(Clone)]
pub struct EventCallback<Message>(
    EventNameString,
    Rc<dyn Fn(&mut dyn RootRender, VdomWeak, web_sys::Event) -> Option<Message>>,
);

impl<Message> PartialEq for EventCallback<Message>
where
    Message: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.0 == other.0 {
            assert_eq!(
                &*self.1 as *const _ as *const u8,
                &*other.1 as *const _ as *const u8
            );

            assert!(std::ptr::eq(
                &*self.1 as *const _ as *const u8,
                &*other.1 as *const _ as *const u8
            ));
        } else {
            assert_ne!(
                &*self.1 as *const _ as *const u8,
                &*other.1 as *const _ as *const u8
            );
            assert!(!std::ptr::eq(
                &*self.1 as *const _ as *const u8,
                &*other.1 as *const _ as *const u8
            ));
        }
        self.0 == other.0
            && std::ptr::eq(
                &*self.1 as *const _ as *const u8,
                &*other.1 as *const _ as *const u8,
            )
    }
}

impl<Message> EventCallback<Message> {
    #[must_use]
    pub fn new(
        name: EventNameString,
        cb: Rc<dyn Fn(&mut dyn RootRender, VdomWeak, web_sys::Event) -> Option<Message>>,
    ) -> Self {
        Self(name, cb)
    }
}

#[derive(Clone)]
pub struct EventMessage<Message>(EventNameString, Rc<dyn Fn() -> Option<Message>>);

impl<Message> EventMessage<Message>
where
    Message: 'static,
{
    /// # Panics
    ///
    /// Will panic if Callback not return  Msg / Option<Msg> or ()
    #[must_use]
    pub fn new<MsU: 'static, F: Fn() -> MsU + 'static>(name: EventNameString, cb: F) -> Self {
        let rc_callback = map_fn_callback_return_to_option_ms!(
            dyn Fn() -> Option<Message>,
            (),
            cb,
            "Callback can return only Msg, Option<Msg> or ()!",
            Rc
        );

        Self(name, rc_callback)
    }
}

impl<Message> PartialEq for EventMessage<Message>
where
    Message: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
        //FIXME comparing trait object pointers compares a non-unique vtable address
        //comparing trait object pointers compares a non-unique vtable address
        // consider extracting and comparing data pointers only
        // for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#vtable_address_comparisons
        //
        //&& Rc::ptr_eq(&self.1, &other.1)
    }
}
#[derive(Clone, PartialEq, From)]
pub enum EventNode<Message> {
    Cb(EventCallback<Message>),
    CbMessage(EventMessage<Message>),
}
impl<Message> Eq for EventNode<Message> where Message: PartialEq {}
// impl<Message> PartialEq for EventNode<Message>
// where
//     Message: PartialEq,
// {
//     fn eq(&self, other: &Self) -> bool {
//         match (self, other) {
//             (Self::Cb(l0), Self::Cb(r0)) => l0 == r0,
//             (Self::CbMessage(l0), Self::CbMessage(r0)) => l0 == r0,
//             (EventNode::Cb(_), EventNode::Cb(_)) => todo!(),
//             (EventNode::Cb(_), EventNode::CbMessage(_)) => todo!(),
//             (EventNode::CbMessage(_), EventNode::Cb(_)) => todo!(),
//             (EventNode::CbMessage(_), EventNode::CbMessage(_)) => todo!(),
//         }
//     }
// }
// impl<Message> From<(EventNameString, Box<dyn EventCbClone>)> for EventNode<Message> {
//     fn from(v: (EventNameString, Box<dyn EventCbClone>)) -> Self {
//         Self::Cb(EventCallback(v.0, v.1))
//     }
// }

// impl<Message> From<(EventNameString, Box<dyn EventMessageCbClone<Message>>)>
//     for EventNode<Message>
// {
//     fn from(v: (EventNameString, Box<dyn EventMessageCbClone<Message>>)) -> Self {
//         Self::CbMessage(EventMessage(v.0, v.1))
//     }
// }

impl<Message> std::fmt::Debug for EventNode<Message>
where
    Message: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventNode::Cb(EventCallback(k, _)) => {
                let v = (k, "Box<dyn EventCbClone>");
                f.debug_tuple("EventNode<Message>").field(&v).finish()
            }
            EventNode::CbMessage(EventMessage(k, _)) => {
                let v = (k, "Box<dyn EventMessageCbClone>");
                f.debug_tuple("EventNode<Message>").field(&v).finish()
            }
        }
    }
}

// #[derive(Clone, Eq)]
// enum BuilderWidget<Message>
// where
//     Message: 'static + PartialEq,
// {
//     Static(Rc<dyn Widget<Message>>),
//     Dyn(Box<dyn DynGElement<Message>>),
// }
// impl<Message> PartialEq for BuilderWidget<Message>
// where
//     Message: 'static + PartialEq,
// {
//     fn eq(&self, other: &Self) -> bool {
//         match (self, other) {
//             (Self::Static(l0), Self::Static(r0)) =>
//             // (&**l0).box_eq((&**r0).as_any())
//             {
//                 (**l0) == (**r0)
//             }

//             (Self::Dyn(l0), Self::Dyn(r0)) => l0 == r0,
//             _ => false,
//         }
//     }
// }

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, DynPartialEq, PartialEq, Eq)]
#[eq_opt(where_add = "Message: PartialEq+'static,")]
pub struct NodeBuilderWidget<Message> {
    id: IdStr,
    //TODO : in areas heap
    pub(crate) widget: Option<Box<GElement<Message>>>,
    //TODO use vec deque
    event_callbacks: VecDeque<EventNode<Message>>,
    // event_callbacks: Vector<EventNode<Message>>,
    layout_str: String,
}

impl<Message> std::fmt::Debug for NodeBuilderWidget<Message>
where
    Message: std::clone::Clone + std::fmt::Debug + PartialEq,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let widget = if self.widget.is_none() {
            String::from("None")
        } else {
            String::from("Option<Rc<dyn NodeBuilder<Message> + 'a>>")
        };

        f.debug_struct("NodeBuilderWidget")
            .field("widget", &widget)
            .field("event_callbacks", &self.event_callbacks)
            .field("layout", &self.layout_str)
            .finish()
    }
}

impl<Message> Default for NodeBuilderWidget<Message> {
    fn default() -> Self {
        Self {
            id: IdStr::new_inline(""),
            widget: None,
            event_callbacks: VecDeque::default(),
            layout_str: String::default(),
        }
    }
}
impl<Message> NodeBuilderWidget<Message> {
    /// # Errors
    ///
    /// Will return `Err` if `gel` does not Layer_(_) | Button_(_) | Text_(_)
    #[allow(clippy::result_unit_err)]
    pub fn try_new_use(gel: &GElement<Message>) -> Result<Self, ()> {
        use GElement::{Button_, Layer_, Text_};
        match gel {
            Layer_(_) | Button_(_) | Text_(_) => Ok(Self::default()), //TODO check if is Generic_
            _ => Err(()),
        }
    }
    pub fn set_id(&mut self, id: IdStr) {
        self.id = id;
    }

    pub fn add_styles_string(&mut self, styles: &str) {
        self.layout_str += styles;
    }
    pub fn add_event_callback(&mut self, event_callback: EventNode<Message>) {
        self.event_callbacks.push_back(event_callback);
    }

    /// Get a reference to the node builder widgets event callbacks.
    #[must_use]
    pub const fn event_callbacks(&self) -> &VecDeque<EventNode<Message>> {
        &self.event_callbacks
    }

    /// Set the node builder widget's widget.
    /// # Panics
    ///
    /// Will Panics if `gel` is Refresher_ | Event_
    /// permission to read it.
    #[must_use]
    pub fn and_widget(mut self, gel: GElement<Message>) -> Self {
        // use match_any::match_any;
        #[allow(unused)]
        use GElement::{
            Builder_, Button_, EmptyNeverUse, Event_, Generic_, Layer_, NodeRef_, Refresher_, Text_,
        };
        let gel_take = gel;

        match &gel_take {
            Builder_(_builder) => {
                // builder.and_widget(*gel_in);
                panic!("check what happened , Builder in builder");
                // FIXME impl NodeBuilder<Message> can set
                // self.widget = Some(BuilderWidget::Static(Rc::new(builder)));
            }
            // Layer_(x) => {
            //     self.widget = Some(BuilderWidget::Static(Rc::new(x) as Rc<dyn Widget<Message>>));
            // }
            // Text_(x) => {
            //     self.widget = Some(BuilderWidget::Static(Rc::new(x) as Rc<dyn Widget<Message>>));
            // }
            // Button_(x) => {
            //     self.widget = Some(BuilderWidget::Static(Rc::new(x) as Rc<dyn Widget<Message>>));
            // }
            Refresher_(_) | Event_(_) => {
                todo!();
            }
            // Generic_(x) => {
            //     debug!("Generic_:: NodeBuilderWidget set widget: Generic_");
            //     self.widget = Some(BuilderWidget::Dyn(x));
            // }
            NodeRef_(_) => panic!("set_widget: GElement::NodeIndex_() should handle before."),

            EmptyNeverUse => panic!("EmptyNeverUse never here"),
            _ => (),
        };
        self.widget = Some(Box::new(gel_take));
        self

        //TODO add type_name
    }

    pub fn widget(&self) -> Option<&Box<GElement<Message>>> {
        self.widget.as_ref()
    }

    pub fn widget_mut(&mut self) -> &mut Option<Box<GElement<Message>>> {
        &mut self.widget
    }
}

impl<Message> Widget<Message> for NodeBuilderWidget<Message>
where
    Message: 'static + Clone + PartialEq,
{
    #[allow(late_bound_lifetime_arguments)]
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
        style_sheet: &GlobalStyleSV,
    ) -> Node<'b> {
        // let mut element_builder = match self.widget.as_ref().unwrap() {
        //     BuilderWidget::Static(x) => x.generate_element_builder(bump, bus, style_sheet),
        //     BuilderWidget::Dyn(x) => x.generate_element_builder(bump, bus, style_sheet),
        // };
        let mut element_builder = self
            .widget()
            .unwrap()
            .as_dyn_node_widget()
            .generate_element_builder(bump, bus, style_sheet);

        debug!("node_builder_widget index:{}", self.id.as_str());

        element_builder = element_builder
            .attr(
                "index",
                bumpalo::collections::String::from_str_in(self.id.as_str(), bump).into_bump_str(),
            )
            .attr(
                "style",
                bumpalo::collections::String::from_str_in(self.layout_str.as_str(), bump)
                    .into_bump_str(),
            );

        // let mut v =
        //     bumpalo::collections::Vec::from_iter_in(self.event_callbacks.iter().cloned(), bump);
        // TODO: `self.event_callbacks`   use take replace the clone
        let mut event_nodes = self.event_callbacks.clone(); //TODO remove clone use ref
                                                            // let mut event_nodes = bumpalo::boxed::Box::new_in(self.event_callbacks.clone(), &bump);

        while let Some(event_node) = event_nodes.pop_front() {
            // let aa = collections::String::from_str_in(event.as_str(), bump);
            // element_builder = element_builder.on(aa.into_bump_str(), callback);
            let event_bus = bus.clone();

            match event_node {
                EventNode::Cb(EventCallback(event, callback)) => {
                    let event_bump_string = {
                        use dodrio::bumpalo::collections::String;
                        String::from_str_in(event.as_str(), bump).into_bump_str()
                    };
                    debug!("{}", &event_bump_string);

                    // element_builder = element_builder.on(event_bump_string, callback);
                    element_builder = element_builder.on(
                        event_bump_string,
                        move |root: &mut dyn RootRender, vdom: VdomWeak, event: web_sys::Event| {
                            if let Some(msg) = callback(root, vdom, event) {
                                event_bus.publish(msg);
                            }
                        },
                    );
                }
                EventNode::CbMessage(EventMessage(event, msg_fn)) => {
                    let event_bump_string = {
                        use dodrio::bumpalo::collections::String;
                        String::from_str_in(event.as_str(), bump).into_bump_str()
                    };
                    debug!("{}", &event_bump_string);

                    element_builder = element_builder.on(
                        event_bump_string,
                        move |_root: &mut dyn RootRender,
                              _vdom: VdomWeak,
                              _event: web_sys::Event| {
                            trace!("borrow_mut g_state_store_refcell");

                            if let Some(msg) = msg_fn() {
                                event_bus.publish(msg);
                            }
                        },
                    );
                }
            }
        }

        element_builder.finish()
    }
}


#[cfg(test)]
#[allow(unused)]
mod node_builder_test {
    use emg_core::vector;
    use iced::Text;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;
    use crate::dodrio::bumpalo::Bump;

    #[derive(Clone)]
    enum Message {
        A,
        B,
    }
    #[wasm_bindgen_test]
    fn test_node_builder() {
        let bump = bumpalo::Bump::new();
        let x = bump.alloc("hello");
        let a =
            |root: &mut dyn RootRender, vdom: VdomWeak, event: web_sys::Event| Option::<i32>::None;

        // let cc = EventCallbackCloneStatic::new(a);

        let a2 =
            |root: &mut dyn RootRender, vdom: VdomWeak, event: web_sys::Event| Option::<i32>::None;
        // let aa2 = fff(a2);

        // let cc2 = EventCallbackCloneStatic::new(a2);

        let f = bump.alloc(|root: &mut dyn RootRender, vdom: VdomWeak, event: web_sys::Event| {});
        let f2 = bump.alloc(
            |root: &mut dyn RootRender, vdom: VdomWeak, event: web_sys::Event| {
                println!("22");
            },
        );

        // let b = NodeBuilderWidget::<'_, Message> {
        //     id: "".to_string(),
        //     widget: Rc::new(Button::new(Text::new("a"))),
        //     event_callbacks: vector![
        //         EventCallback(String::from("xxx"), Box::new((a))).into(),
        //         EventNode::Cb(EventCallback(String::from("ff"), Box::new((a2)))),
        //         EventMessage(String::from("x"), Box::new(|| Message::A)).into(),
        //     ],
        //     layout_str: String::default(),
        // };
    }
}
