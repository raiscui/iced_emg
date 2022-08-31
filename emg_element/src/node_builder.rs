/*
 * @Author: Rais
 * @Date: 2022-08-18 18:05:52
 * @LastEditTime: 2022-08-31 15:19:21
 * @LastEditors: Rais
 * @Description:
 */
/*
 * @Author: Rais
 * @Date: 2021-03-08 18:20:22
 * @LastEditTime: 2022-08-18 10:58:06
 * @LastEditors: Rais
 * @Description:
 */

#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::ptr_eq)]
// ────────────────────────────────────────────────────────────────────────────────

use derive_more::From;

use emg_common::{na::Translation3, IdStr, NotNan};
use emg_layout::{EdgeCtx, LayoutEndType};
use emg_native::{WidgetState, DPR};
use emg_refresh::RefreshForUse;
use emg_state::{StateAnchor, StateMultiAnchor};
use tracing::{debug, info, info_span, instrument, trace};

use crate::{widget::Widget, GElement};
use std::{collections::VecDeque, rc::Rc, string::String};

type EventNameString = IdStr;

// Rc<dyn Fn(&mut dyn RootRender, VdomWeak, web_sys::Event) -> Option<Message>>;
type EventCallbackFn<Message> = Rc<dyn Fn(&mut i32) -> Option<Message>>;

pub struct EventCallback<Message>(EventNameString, EventCallbackFn<Message>);

impl<Message> Clone for EventCallback<Message> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

impl<Message> PartialEq for EventCallback<Message>
// where
//     Message: PartialEq,
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
    pub fn new(name: EventNameString, cb: EventCallbackFn<Message>) -> Self {
        Self(name, cb)
    }
}

pub struct EventMessage<Message>(EventNameString, Rc<dyn Fn() -> Option<Message>>);

impl<Message> Clone for EventMessage<Message> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

impl<Message> EventMessage<Message>
where
    Message: 'static,
{
    /// # Panics
    ///
    /// Will panic if Callback not return  Msg / Option<Msg> or ()
    #[must_use]
    pub fn new<MsU: 'static, F: Fn() -> MsU + 'static>(name: EventNameString, cb: F) -> Self {
        todo!()
        // let rc_callback = map_fn_callback_return_to_option_ms!(
        //     dyn Fn() -> Option<Message>,
        //     (),
        //     cb,
        //     "Callback can return only Msg, Option<Msg> or ()!",
        //     Rc
        // );

        // Self(name, rc_callback)
    }
}

impl<Message> PartialEq for EventMessage<Message>
// where
//     Message: PartialEq,
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
#[derive(From)]
pub enum EventNode<Message> {
    Cb(EventCallback<Message>),
    CbMessage(EventMessage<Message>),
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
// where
//     Message: std::fmt::Debug,
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
pub struct NodeBuilderWidget<Message, RenderContext> {
    id: IdStr,
    //TODO : in areas heap
    widget: Box<GElement<Message, RenderContext>>,
    //TODO use vec deque
    event_callbacks: VecDeque<EventNode<Message>>,
    // event_callbacks: Vector<EventNode<Message>>,
    // layout_str: String,
    // layout_end: (Translation3<NotNan<f64>>, NotNan<f64>, NotNan<f64>),
    widget_state: StateAnchor<WidgetState>,
}

impl<Message, RenderContext> PartialEq for NodeBuilderWidget<Message, RenderContext> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.widget == other.widget
            && self.event_callbacks == other.event_callbacks
            && self.widget_state == other.widget_state
        // && self.layout_str == other.layout_str
    }
}

impl<Message, RenderContext> Clone for NodeBuilderWidget<Message, RenderContext> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            widget: self.widget.clone(),
            event_callbacks: self.event_callbacks.clone(),
            widget_state: self.widget_state.clone(), // layout_str: self.layout_str.clone(),
        }
    }
}

impl<Message, RenderContext> std::fmt::Debug for NodeBuilderWidget<Message, RenderContext>
// where
//     Message: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let widget = if self.widget.is_none() {
        //     String::from("None")
        // } else {
        //     String::from("Option<Rc<dyn NodeBuilder<Message> + 'a>>")
        // };
        let widget = String::from("GElement<Message>");

        f.debug_struct("NodeBuilderWidget")
            .field("widget", &widget)
            .field("event_callbacks", &self.event_callbacks)
            .field("widget_state", &self.widget_state)
            .finish()
    }
}

// impl<Message> Default for NodeBuilderWidget<Message> {
//     fn default() -> Self {
//         Self {
//             id: IdStr::new_inline(""),
//             widget: None,
//             event_callbacks: VecDeque::default(),
//             layout_str: String::default(),
//         }
//     }
// }
impl<Message, RenderCtx> NodeBuilderWidget<Message, RenderCtx>
where
    RenderCtx: 'static,
{
    fn new(gel: GElement<Message, RenderCtx>, edge_ctx: &StateAnchor<EdgeCtx<RenderCtx>>) -> Self {
        //TODO check in debug , combine  use  try_new_use
        match &gel {
            // Builder_(_builder) => {
            //     // builder.and_widget(*gel_in);
            //     panic!("check what happened , Builder in builder");
            //     // FIXME impl NodeBuilder<Message> can set
            //     // self.widget = Some(BuilderWidget::Static(Rc::new(builder)));
            // }
            // Layer_(x) => {
            //     self.widget = Some(BuilderWidget::Static(Rc::new(x) as Rc<dyn Widget<Message>>));
            // }
            // Text_(x) => {
            //     self.widget = Some(BuilderWidget::Static(Rc::new(x) as Rc<dyn Widget<Message>>));
            // }
            // Button_(x) => {
            //     self.widget = Some(BuilderWidget::Static(Rc::new(x) as Rc<dyn Widget<Message>>));
            // }
            GElement::Refresher_(_) => {
                todo!();
            }
            // Generic_(x) => {
            //     debug!("Generic_:: NodeBuilderWidget set widget: Generic_");
            //     self.widget = Some(BuilderWidget::Dyn(x));
            // }
            GElement::NodeRef_(_) => {
                panic!("set_widget: GElement::NodeIndex_() should handle before.")
            }

            GElement::EmptyNeverUse => panic!("EmptyNeverUse never here"),
            _ => (),
        };
        //     //TODO add type_name
        //TODO setup real id in build

        let widget_state = edge_ctx.then(
            |EdgeCtx {
                 styles_end,
                 layout_end,
                 ..
             }| {
                (styles_end, layout_end)
                    .map(|styles, &(trans, w, h)| {
                        let new_widget_state = WidgetState::new((w, h), trans);
                        styles.values().fold(new_widget_state, |mut ws, x| {
                            x.refresh_for(&mut ws);
                            ws
                        })
                    })
                    .into_anchor()
            },
        );
        // let widget_state = layout_end.map(|&(trans, w, h)| WidgetState::new((w, h), trans));

        Self {
            id: IdStr::new_inline(""),
            widget: Box::new(gel),
            event_callbacks: VecDeque::default(),
            // layout_str: String::default(),
            widget_state,
        }
    }
    /// # Errors
    ///
    /// Will return `Err` if `gel` does not Layer_(_) | Button_(_) | Text_(_)
    /// # Panics
    ///
    /// Will panic if xxxx
    // #[allow(clippy::match_same_arms)]
    // pub fn try_new_use(gel: &GElement<Message>) -> Result<Self, ()> {
    //     use GElement::{Button_, Layer_, Text_};
    //     match gel {
    //         Layer_(_) | Button_(_) | Text_(_) | GElement::Generic_(_) => Ok(Self::default()),
    //         GElement::Builder_(_) => panic!("crate builder use builder is not supported"),
    //         GElement::Refresher_(_) | GElement::Event_(_) => Err(()),
    //         GElement::NodeRef_(_) => {
    //             unreachable!("crate builder use NodeRef_ is should never happened")
    //         }

    //         GElement::EmptyNeverUse => {
    //             unreachable!("crate builder use EmptyNeverUse is should never happened")
    //         }
    //         GElement::SaNode_(_) => todo!(),
    //         GElement::EvolutionaryFactor(_) => todo!(),
    //     }
    // }

    //TODO use try into
    #[allow(clippy::match_same_arms)]
    pub fn try_new_use(
        gel: GElement<Message, RenderCtx>, //TODO use anchor instead
        edge_ctx: &StateAnchor<EdgeCtx<RenderCtx>>,
    ) -> Result<Self, GElement<Message, RenderCtx>> {
        match gel {
            // Layer_(_) | Button_(_) | Text_(_) | GElement::Generic_(_) => Ok(Self::default()),
            GElement::Layer_(_) | GElement::Generic_(_) => Ok(Self::new(gel, edge_ctx)),
            GElement::Builder_(_) => panic!("crate builder use builder is not supported"),
            // GElement::Refresher_(_) | GElement::Event_(_) => Err(()),
            GElement::Refresher_(_) => Err(gel),
            GElement::NodeRef_(_) => {
                unreachable!("crate builder use NodeRef_ is should never happened")
            }

            GElement::EmptyNeverUse => {
                unreachable!("crate builder use EmptyNeverUse is should never happened")
            }
            GElement::SaNode_(_) => todo!(),
            GElement::EvolutionaryFactor(_) => todo!(),
        }
    }
    pub fn set_id(&mut self, id: IdStr) {
        self.id = id;
    }

    // pub fn add_styles_string(&mut self, styles: &str) {
    //     self.layout_str += styles;
    // }
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
    // #[must_use]
    // pub fn and_widget(mut self, gel: GElement<Message>) -> Self {
    //     // use match_any::match_any;
    //     #[allow(unused)]
    //     use GElement::{
    //         Builder_, Button_, EmptyNeverUse, Event_, Generic_, Layer_, NodeRef_, Refresher_, Text_,
    //     };
    //     let gel_take = gel;
    //     //TODO check in debug , combine  use  try_new_use
    //     match &gel_take {
    //         Builder_(_builder) => {
    //             // builder.and_widget(*gel_in);
    //             panic!("check what happened , Builder in builder");
    //             // FIXME impl NodeBuilder<Message> can set
    //             // self.widget = Some(BuilderWidget::Static(Rc::new(builder)));
    //         }
    //         // Layer_(x) => {
    //         //     self.widget = Some(BuilderWidget::Static(Rc::new(x) as Rc<dyn Widget<Message>>));
    //         // }
    //         // Text_(x) => {
    //         //     self.widget = Some(BuilderWidget::Static(Rc::new(x) as Rc<dyn Widget<Message>>));
    //         // }
    //         // Button_(x) => {
    //         //     self.widget = Some(BuilderWidget::Static(Rc::new(x) as Rc<dyn Widget<Message>>));
    //         // }
    //         Refresher_(_) | Event_(_) => {
    //             todo!();
    //         }
    //         // Generic_(x) => {
    //         //     debug!("Generic_:: NodeBuilderWidget set widget: Generic_");
    //         //     self.widget = Some(BuilderWidget::Dyn(x));
    //         // }
    //         NodeRef_(_) => panic!("set_widget: GElement::NodeIndex_() should handle before."),

    //         EmptyNeverUse => panic!("EmptyNeverUse never here"),
    //         _ => (),
    //     };
    //     self.widget = Some(Box::new(gel_take));
    //     self

    //     //TODO add type_name
    // }

    #[must_use]
    #[allow(clippy::borrowed_box)]
    pub const fn widget(&self) -> &GElement<Message, RenderCtx> {
        //TODO use cow/beef

        &self.widget
    }

    #[must_use]
    #[allow(clippy::borrowed_box)]
    pub fn get_widget(self) -> Box<GElement<Message, RenderCtx>> {
        //TODO use cow/beef

        self.widget
    }

    pub fn widget_mut(&mut self) -> &mut GElement<Message, RenderCtx> {
        &mut self.widget
    }
}

#[cfg(all(feature = "gpu"))]
impl<Message, RenderContext> crate::Widget<Message, RenderContext>
    for NodeBuilderWidget<Message, RenderContext>
where
    RenderContext: emg_native::RenderContext + Clone + PartialEq + 'static,
    Message: 'static,
    // Message: PartialEq + 'static + std::clone::Clone,
{
    // #[instrument(skip(ctx), name = "NodeBuilderWidget paint")]
    // fn paint(&self, ctx: &mut crate::PaintCtx<RenderContext>) {
    //     ctx.set_widget_state(self.widget_state);
    //     ctx.with_save(|ctx| {
    //         info!(
    //             "NodeBuilderWidget::render: translate: {:?}",
    //             (
    //                 self.widget_state.translation.x,
    //                 self.widget_state.translation.y,
    //             )
    //         );
    //         ctx.transform(emg_native::Affine::translate((
    //             self.widget_state.translation.x * DPR,
    //             self.widget_state.translation.y * DPR,
    //         )));

    //         self.widget().paint(ctx);
    //     });
    // }
    // #[instrument(skip(ctx), name = "NodeBuilderWidget [paint]")]
    // fn paint(&self, ctx: &mut crate::PaintCtx<RenderContext>) {
    //     panic!("NodeBuilderWidget no paint");
    // }

    #[instrument(skip(self, ctx), name = "NodeBuilderWidget paint")]
    fn paint_sa(
        &self,
        ctx: StateAnchor<crate::PaintCtx<RenderContext>>,
    ) -> StateAnchor<crate::PaintCtx<RenderContext>> {
        let id1 = self.id.clone();
        let id2 = self.id.clone();
        let span1 = info_span!("NodeBuilderWidget::paint_sa", id = %self.id);
        let span2 = span1.clone();
        let span3 = span1.clone();

        let current_ctx = (&ctx, &self.widget_state).map(move |incoming_ctx, widget_state| {
            // let id = id.clone();
            let _span = span1.clone().entered();
            info!(
                parent: &span1,
                "NodeBuilderWidget::paint-> (&ctx, &self.widget_state).map -> recalculating [{}]",
                &id1
            );
            let mut incoming_ctx_mut = incoming_ctx.clone();
            incoming_ctx_mut.save();
            incoming_ctx_mut.set_widget_state(widget_state.clone());
            incoming_ctx_mut.transform(emg_native::Affine::translate((
                widget_state.translation.x * DPR,
                widget_state.translation.y * DPR,
            )));
            incoming_ctx_mut
        });
        illicit::Layer::new()
            .offer(span3)
            .enter(|| self.widget.paint_sa(current_ctx))
            .map(move |out_ctx| {
                info!(
                    parent: &span2,
                    " widget.paint end -> recalculating restore [{}]", &id2
                );
                let mut out_ctx_mut = out_ctx.clone();
                out_ctx_mut.restore();
                out_ctx_mut
            })
    }
}

#[cfg(target_arch = "wasm32")]
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

#[cfg(all(test, target_arch = "wasm32"))]
#[allow(unused)]
mod node_builder_test {
    use emg_common::vector;
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
