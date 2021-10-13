/*
 * @Author: Rais
 * @Date: 2021-03-08 18:20:22
 * @LastEditTime: 2021-10-12 13:32:07
 * @LastEditors: Rais
 * @Description:
 */

mod gelement2nodebuilderwidget;
// use dyn_clone::DynClone;
// ────────────────────────────────────────────────────────────────────────────────

use derive_more::From;
use seed_styles::GlobalStyleSV;
use tracing::warn;

use std::{cell::RefCell, rc::Rc, string::String};

use crate::{
    dodrio::{
        self, builder::ElementBuilder, bumpalo, Attribute, Listener, Node, RootRender, VdomWeak,
    },
    map_fn_callback_return_to_option_ms, Bus, DynGElement, Element, GElement, Widget,
};
use emg_core::Vector;
// ────────────────────────────────────────────────────────────────────────────────
//TODO move out to global
pub trait NodeBuilder<Message> // where
// Message: 'static,
{
    fn generate_element_builder<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
        style_sheet: &GlobalStyleSV,
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
type EventNameString = String;

#[derive(Clone)]
pub struct EventCallback<Message>(
    EventNameString,
    Rc<dyn Fn(&mut dyn RootRender, VdomWeak, web_sys::Event) -> Option<Message>>,
);

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
#[derive(Clone, From)]
pub enum EventNode<Message> {
    Cb(EventCallback<Message>),
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

#[derive(Clone)]
enum BuilderWidget<Message>
where
    Message: 'static,
{
    Static(Rc<dyn NodeBuilder<Message>>),
    Dyn(Box<dyn DynGElement<Message>>),
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct NodeBuilderWidget<Message>
where
    Message: 'static,
{
    id: String,
    //TODO : instead use GElement
    widget: Option<BuilderWidget<Message>>,
    //TODO use vec deque
    event_callbacks: Vector<EventNode<Message>>,
    // event_callbacks: Vector<EventNode<Message>>,
    layout_str: String,
}

impl<Message> std::fmt::Debug for NodeBuilderWidget<Message>
where
    Message: std::clone::Clone + std::fmt::Debug,
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

impl<Message: Clone + 'static> Default for NodeBuilderWidget<Message> {
    fn default() -> Self {
        Self {
            id: String::default(),
            widget: None,
            event_callbacks: Vector::default(),
            layout_str: String::default(),
        }
    }
}
impl<Message: std::clone::Clone + 'static> NodeBuilderWidget<Message> {
    /// # Errors
    ///
    /// Will return `Err` if `gel` does not Layer_(_) | Button_(_) | Text_(_)
    #[allow(clippy::result_unit_err)]
    pub fn try_new_use(gel: &GElement<Message>) -> Result<Self, ()> {
        use GElement::{Button_, Layer_, Text_};
        match gel {
            Layer_(_) | Button_(_) | Text_(_) => Ok(Self::default()),
            _ => Err(()),
        }
    }
    pub fn set_id(&mut self, id: String) {
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
    pub fn event_callbacks(&self) -> &Vector<EventNode<Message>> {
        &self.event_callbacks
    }

    /// Set the node builder widget's widget.
    /// # Panics
    ///
    /// Will Panics if `gel` is Refresher_ | Event_
    /// permission to read it.
    pub fn set_widget(&mut self, gel: &Rc<RefCell<GElement<Message>>>) {
        // use match_any::match_any;
        use GElement::{Builder_, Button_, Event_, Generic_, Layer_, NodeRef_, Refresher_, Text_};
        let gel_take = gel.replace(GElement::NodeRef_(std::string::String::default()));
        match gel_take {
            Builder_(ref gel_in, mut builder) => {
                builder.set_widget(gel_in);
            }
            Layer_(x) => {
                self.widget = Some(BuilderWidget::Static(Rc::new(x)));
            }
            Text_(x) => {
                self.widget = Some(BuilderWidget::Static(Rc::new(x)));
            }
            Button_(x) => {
                self.widget = Some(BuilderWidget::Static(Rc::new(x)));
            }
            Refresher_(_) | Event_(_) => {
                todo!();
            }
            Generic_(x) => {
                self.widget = Some(BuilderWidget::Dyn(x));
            }
            NodeRef_(_) => panic!("set_widget: GElement::NodeIndex_() should handle before."),
        };

        //TODO add type_name
    }
}

impl<Message> Widget<Message> for NodeBuilderWidget<Message>
where
    Message: 'static + Clone,
{
    #[allow(late_bound_lifetime_arguments)]
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
        style_sheet: &GlobalStyleSV,
    ) -> Node<'b> {
        let mut element_builder = match self.widget.as_ref().unwrap() {
            BuilderWidget::Static(x) => x.generate_element_builder(bump, bus, style_sheet),
            BuilderWidget::Dyn(x) => x.generate_element_builder(bump, bus, style_sheet),
        };

        warn!("node_builder_widget index:{}", self.id.as_str());

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
        let mut event_nodes = self.event_callbacks.clone();
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

                    element_builder = element_builder.on(
                        event_bump_string,
                        move |_root: &mut dyn RootRender,
                              _vdom: VdomWeak,
                              _event: web_sys::Event| {
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

impl<Message> From<NodeBuilderWidget<Message>> for Element<Message>
where
    Message: 'static + Clone,
{
    fn from(node_builder_widget: NodeBuilderWidget<Message>) -> Self {
        Self::new(node_builder_widget)
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
