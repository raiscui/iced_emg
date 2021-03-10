// use dyn_clone::DynClone;
use std::rc::Rc;

use iced_web::{
    dodrio::{builder::ElementBuilder, bumpalo, Attribute, Listener, Node, RootRender, VdomWeak},
    Bus, Css, Widget,
};

/*
 * @Author: Rais
 * @Date: 2021-03-08 18:20:22
 * @LastEditTime: 2021-03-10 13:44:04
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

pub trait EventCallbackClone: Fn(&mut dyn RootRender, VdomWeak, web_sys::Event) {
    fn clone_box(&self) -> Box<dyn EventCallbackClone>;
}

impl<T> EventCallbackClone for T
where
    T: 'static + Fn(&mut dyn RootRender, VdomWeak, web_sys::Event) + Clone,
{
    fn clone_box(&self) -> Box<dyn EventCallbackClone> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn EventCallbackClone> {
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
#[derive(Clone)]
struct NodeBuilderWidget<'a, Message> {
    pub(crate) widget: Rc<dyn NodeBuilder<Message> + 'a>,
    event_callbacks: Vec<(String, Box<dyn EventCallbackClone>)>,
}

fn take<T>(vec: &mut Vec<T>, index: usize) -> Option<T> {
    // fn take<T>(mut vec: iced_web::dodrio::bumpalo::collections::Vec<T>, index: usize) -> Option<T> {
    if index < vec.len() {
        Some(vec.swap_remove(index))
    } else {
        None
    }
}

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

        let mut event_callbacks = self.event_callbacks.clone();

        while let Some((event, callback)) = take(&mut event_callbacks, 0) {
            // let aa = collections::String::from_str_in(event.as_str(), bump);
            // element_builder = element_builder.on(aa.into_bump_str(), callback);
            element_builder = element_builder.on(bump.alloc(event), callback);
        }

        element_builder.finish()
    }
}
#[cfg(test)]
#[allow(unused)]
mod node_builder_test {
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
            event_callbacks: vec![
                (String::from("xxx"), Box::new(a)),
                (String::from("ff"), Box::new(a2)),
            ],
        };
    }
}
