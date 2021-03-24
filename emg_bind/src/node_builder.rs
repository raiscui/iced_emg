// use dyn_clone::DynClone;
use derive_more::From;

use std::{convert::TryFrom, rc::Rc};

use crate::GElement;
use emg::im::Vector;
use iced::Element;
use iced_web::{
    dodrio::{
        self, builder::ElementBuilder, bumpalo, Attribute, Listener, Node, RootRender, VdomWeak,
    },
    Bus, Css, Widget,
};

/*
 * @Author: Rais
 * @Date: 2021-03-08 18:20:22
 * @LastEditTime: 2021-03-13 16:24:37
 * @LastEditors: Rais
 * @Description:
 */
pub trait NodeBuilder<Message>
where
    Message: 'static + Clone,
{
    fn make_element_builder<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
        style_sheet: &mut Css<'b>,
    ) -> ElementBuilder<
        'b,
        bumpalo::collections::Vec<'b, Listener<'b>>,
        bumpalo::collections::Vec<'b, Attribute<'b>>,
        bumpalo::collections::Vec<'b, Node<'b>>,
    >;
}
// pub type ListenerCallback = Box<dyn EventCallbackClone + 'static>;

// pub trait EventCallbackClone: Fn(&mut dyn RootRender, VdomWeak, web_sys::Event) + DynClone {}
// dyn_clone::clone_trait_object!(EventCallbackClone);

// ────────────────────────────────────────────────────────────────────────────────

pub trait EventCbClone: Fn(&mut dyn RootRender, VdomWeak, web_sys::Event) {
    fn clone_box(&self) -> Box<dyn EventCbClone>;
}

impl<T> EventCbClone for T
where
    T: 'static + Fn(&mut dyn RootRender, VdomWeak, web_sys::Event) + Clone,
{
    fn clone_box(&self) -> Box<dyn EventCbClone> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn EventCbClone> {
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}
// ────────────────────────────────────────────────────────────────────────────────

pub trait EventMessageCbClone<Message>: Fn() -> Message {
    fn clone_box(&self) -> Box<dyn EventMessageCbClone<Message>>;
}

impl<Message, T> EventMessageCbClone<Message> for T
where
    T: 'static + Fn() -> Message + Clone,
{
    fn clone_box(&self) -> Box<dyn EventMessageCbClone<Message>> {
        Box::new(self.clone())
    }
}

impl<Message> Clone for Box<dyn EventMessageCbClone<Message>> {
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}
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
type EventNameString = String;

#[derive(Clone)]
pub struct EventCallback(EventNameString, Box<dyn EventCbClone>);

impl EventCallback {
    #[must_use]
    pub fn new(name: EventNameString, cb: Box<dyn EventCbClone>) -> Self {
        Self(name, cb)
    }
}

#[derive(Clone)]
pub struct EventMessage<Message>(EventNameString, Box<dyn EventMessageCbClone<Message>>);
impl<Message> EventMessage<Message> {
    #[must_use]
    pub fn new(name: EventNameString, message: Box<dyn EventMessageCbClone<Message>>) -> Self {
        Self(name, message)
    }
}
#[derive(Clone, From)]
pub enum EventNode<Message> {
    Cb(EventCallback),
    CbMessage(EventMessage<Message>),
}

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
    Message: std::clone::Clone + std::fmt::Debug,
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

#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct NodeBuilderWidget<'a, Message> {
    //TODO : instead use GElement
    widget: Rc<dyn NodeBuilder<Message> + 'a>,
    event_callbacks: Vector<EventNode<Message>>,
}

impl<'a, Message> std::fmt::Debug for NodeBuilderWidget<'a, Message>
where
    Message: std::clone::Clone + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeBuilderWidget")
            .field("widget", &String::from("Rc<dyn NodeBuilder<Message> + 'a>"))
            .field("event_callbacks", &self.event_callbacks())
            .finish()
    }
}

impl<'a, Message: std::clone::Clone> NodeBuilderWidget<'a, Message> {
    pub fn new(widget: Rc<dyn NodeBuilder<Message> + 'a>) -> Self {
        Self {
            widget,
            event_callbacks: Vector::new(),
        }
    }
    pub fn add_event_callback(&mut self, event_callback: EventNode<Message>) {
        self.event_callbacks.push_back(event_callback);
    }

    /// Get a reference to the node builder widgets event callbacks.
    #[must_use]
    pub fn event_callbacks(&self) -> &Vector<EventNode<Message>> {
        &self.event_callbacks
    }
}

impl<'a, Message> TryFrom<GElement<'a, Message>> for NodeBuilderWidget<'a, Message>
where
    Message: 'static + Clone,
{
    type Error = GElement<'a, Message>;

    fn try_from(gel: GElement<'a, Message>) -> Result<Self, Self::Error> {
        use match_any::match_any;
        use GElement::{Button_, Layer_};
        match_any! (gel,
            Layer_( x) |Button_(x)=> {
                Ok(NodeBuilderWidget::new(Rc::new(x)))
            },
            _=>Err(gel)
        )
    }
}

// TODO move to utilities
// fn take<T>(vec: &mut Vec<T>, index: usize) -> Option<T> {
//     // fn take<T>(mut vec: iced_web::dodrio::bumpalo::collections::Vec<T>, index: usize) -> Option<T> {
//     if index < vec.len() {
//         Some(vec.swap_remove(index))
//     } else {
//         None
//     }
// }

impl<'a, Message> Widget<Message> for NodeBuilderWidget<'a, Message>
where
    Message: 'static + Clone,
{
    #[allow(late_bound_lifetime_arguments)]
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
        style_sheet: &mut Css<'b>,
    ) -> Node<'b> {
        let mut element_builder = self.widget.make_element_builder(bump, bus, style_sheet);

        // let mut v =
        //     bumpalo::collections::Vec::from_iter_in(self.event_callbacks.iter().cloned(), bump);

        let mut event_nodes = self.event_callbacks.clone();

        while let Some(event_node) = event_nodes.pop_front() {
            // let aa = collections::String::from_str_in(event.as_str(), bump);
            // element_builder = element_builder.on(aa.into_bump_str(), callback);

            match event_node {
                EventNode::Cb(EventCallback(event, callback)) => {
                    let event_bump_string = {
                        use dodrio::bumpalo::collections::String;
                        String::from_str_in(event.as_str(), bump).into_bump_str()
                    };
                    element_builder = element_builder.on(event_bump_string, callback);
                }
                EventNode::CbMessage(EventMessage(event, msg)) => {
                    let event_bump_string = {
                        use dodrio::bumpalo::collections::String;
                        String::from_str_in(event.as_str(), bump).into_bump_str()
                    };
                    let event_bus = bus.clone();

                    element_builder = element_builder.on(
                        event_bump_string,
                        move |_root: &mut dyn RootRender,
                              _vdom: VdomWeak,
                              _event: web_sys::Event| {
                            event_bus.publish(msg());
                        },
                    );
                }
            }
        }

        element_builder.finish()
    }
}

impl<'a, Message> From<NodeBuilderWidget<'a, Message>> for Element<'a, Message>
where
    Message: 'static + Clone,
{
    fn from(node_builder_widget: NodeBuilderWidget<'a, Message>) -> Element<'a, Message> {
        Element::new(node_builder_widget)
    }
}
#[cfg(test)]
#[allow(unused)]
mod node_builder_test {
    use emg::im::vector;
    use iced::Text;
    use wasm_bindgen_test::*;

    use crate::Button;

    use super::*;
    use iced_web::dodrio::bumpalo::Bump;

    #[derive(Clone)]
    enum Message {
        A,
        B,
    }
    #[wasm_bindgen_test]
    fn test_node_builder() {
        let bump = bumpalo::Bump::new();
        let x = bump.alloc("hello");
        let a = |root: &mut dyn RootRender, vdom: VdomWeak, event: web_sys::Event| {};

        // let cc = EventCallbackCloneStatic::new(a);

        let a2 = |root: &mut dyn RootRender, vdom: VdomWeak, event: web_sys::Event| {};
        // let aa2 = fff(a2);

        // let cc2 = EventCallbackCloneStatic::new(a2);

        let f = bump.alloc(|root: &mut dyn RootRender, vdom: VdomWeak, event: web_sys::Event| {});
        let f2 = bump.alloc(
            |root: &mut dyn RootRender, vdom: VdomWeak, event: web_sys::Event| {
                println!("22");
            },
        );

        let b = NodeBuilderWidget::<'_, Message> {
            widget: Rc::new(Button::new(Text::new("a"))),
            event_callbacks: vector![
                EventCallback(String::from("xxx"), Box::new((a))).into(),
                EventNode::Cb(EventCallback(String::from("ff"), Box::new((a2)))),
                EventMessage(String::from("x"), Box::new(|| Message::A)).into(),
            ],
        };
    }
}
