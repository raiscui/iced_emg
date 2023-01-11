/*
 * @Author: Rais
 * @Date: 2022-08-18 18:05:52
 * @LastEditTime: 2023-01-11 16:02:50
 * @LastEditors: Rais
 * @Description:
 */

#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::ptr_eq)]
// ────────────────────────────────────────────────────────────────────────────────
mod event_builder;
use derive_more::From;

use emg_common::{mouse, na::Translation3, vector, IdStr, NotNan, Pos, TypeName, Vector};
use emg_layout::{EdgeCtx, LayoutEndType};
use emg_native::{
    event::EventFlag, paint_ctx::CtxIndex, renderer::Rect, Event, WidgetState, DPR, G_POS,
};
use emg_shaping::{EqShapingWithDebug, ShapeOfUse, Shaping, ShapingUse};
use emg_state::{Anchor, Dict, StateAnchor, StateMultiAnchor};
use tracing::{debug, debug_span, info, info_span, instrument, trace, Span};

use crate::{map_fn_callback_return_to_option_ms, widget::Widget, GElement};
use std::{cell::Cell, collections::VecDeque, rc::Rc, string::String};

use self::event_builder::EventListener;

/// EventIdentify(emg_native::event::EventFlag::X,X::EventFlag)
/// EventIdentify(Level1,Level2)
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

// Rc<dyn Fn(&mut dyn RootRender, VdomWeak, web_sys::Event) -> Option<Message>>;
type EventCallbackFn<Message> = Rc<dyn Fn(&mut i32) -> Option<Message>>;

pub struct EventCallback<Message>(EventIdentify, EventCallbackFn<Message>);

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
    pub fn new(name: EventIdentify, cb: EventCallbackFn<Message>) -> Self {
        Self(name, cb)
    }
}

pub struct EventMessage<Message>(EventIdentify, Rc<dyn Fn() -> Option<Message>>);

impl<Message> Clone for EventMessage<Message> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
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
        None
    }
}

impl<Message> IntoOptionMs<Message> for Option<Message> {
    fn into_option(self) -> Option<Message> {
        self
    }
}
// ────────────────────────────────────────────────────────────────────────────────

impl<Message: 'static> EventMessage<Message> {
    pub fn new<T: IntoOptionMs<Message> + 'static>(
        name: EventIdentify,
        cb: impl Fn() -> T + 'static,
        // cb: impl FnOnce() -> T + Clone + 'static,
    ) -> Self {
        Self(name, Rc::new(move || cb().into_option()))
    }
}

// impl<Message> EventMessage<Message>
// where
//     Message: 'static,
// {
//     #[must_use]
//     pub fn new<MsU: 'static, F: Fn() -> MsU + 'static>(name: EventNameString, cb: F) -> Self {
//         let rc_callback = map_fn_callback_return_to_option_ms!(
//             dyn Fn() -> Option<Message>,
//             (),
//             cb,
//             "Callback can return only Msg, Option<Msg> or ()!",
//             Rc
//         );
//         Self(name, rc_callback)
//     }
// }

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
        // && Rc::ptr_eq(&self.1, &other.1)
    }
}
#[derive(From)]
pub enum EventNode<Message> {
    Cb(EventCallback<Message>),
    CbMessage(EventMessage<Message>),
}

impl<Message> EventNode<Message> {
    pub fn get_identify(&self) -> EventIdentify {
        match self {
            EventNode::Cb(x) => x.0.clone(),
            EventNode::CbMessage(x) => x.0.clone(),
        }
    }
    pub fn call(&self) {
        match self {
            EventNode::Cb(x) => {
                info!("EventNode::Cb call");
                (x.1)(&mut 1);
            }
            EventNode::CbMessage(x) => {
                (x.1)();
            }
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
pub struct NodeBuilderWidget<Message> {
    id: IdStr,
    //TODO : in areas heap
    widget: Box<GElement<Message>>,
    //TODO use vec deque
    // event_callbacks: VecDeque<EventNode<Message>>,
    event_listener: EventListener<Message>,
    // event_callbacks: Vector<EventNode<Message>>,
    // layout_str: String,
    // layout_end: (Translation3<NotNan<f64>>, NotNan<f64>, NotNan<f64>),
    widget_state: StateAnchor<WidgetState>,
}

impl<Message> PartialEq for NodeBuilderWidget<Message> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.widget == other.widget
            // && self.event_callbacks == other.event_callbacks
            && self.widget_state == other.widget_state
            && self.event_listener == other.event_listener
        // && self.layout_str == other.layout_str
    }
}

impl<Message> Clone for NodeBuilderWidget<Message> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            widget: self.widget.clone(),
            widget_state: self.widget_state.clone(),
            event_listener: self.event_listener.clone(),
        }
    }
}

impl<Message> std::fmt::Debug for NodeBuilderWidget<Message>
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
            .field("event_listener", &self.event_listener)
            .field("widget_state", &self.widget_state)
            .finish()
    }
}

pub type EventMatchsDict<Message> = Dict<EventIdentify, (Event, Vector<EventNode<Message>>)>;

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
impl<Message> NodeBuilderWidget<Message>
where
    Message: 'static,
{
    fn new(ix: &IdStr, gel: GElement<Message>, edge_ctx: &StateAnchor<EdgeCtx>) -> Self {
        #[cfg(debug_assertions)]
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
            GElement::Shaper_(_) => {
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
                 world,
                 children_layout_override,
                 ..
             }| {
                let world_clone = world.clone();
                let children_layout_override_clone = children_layout_override.clone();
                let styles_sa = styles_end.map_(|_k, v| v.get_anchor()).then(|x| {
                    x.clone()
                        .into_iter()
                        .collect::<Anchor<Dict<TypeName, Rc<dyn EqShapingWithDebug<WidgetState>>>>>(
                        )
                });

                //TODO 不要用 顺序pipe , 这样情况下 size trans改变 会 重新 进行全部 style 计算,使用 mut 保存 ws.
                layout_end
                    .map(move |&(trans, w, h)| {
                        WidgetState::new(
                            (w, h),
                            trans,
                            world_clone.clone(),
                            children_layout_override_clone.clone(),
                        )
                    })
                    .then(move |ws| {
                        styles_sa
                            .increment_reduction(ws.clone(), |out_ws, _k, v| {
                                debug!("increment_reduction ------  {:?}", v);
                                v.as_ref().shaping(out_ws);
                                // out_ws.shaping_use(v.as_ref());
                                // out_ws.shape_of_use(v.as_ref() as &dyn Shaping<WidgetState>);
                            })
                            .into_anchor()
                    })
                    .into_anchor()

                // (styles_end, layout_end)
                //     .map(move |styles, &(trans, w, h)| {
                //         let new_widget_state = WidgetState::new(
                //             (w, h),
                //             trans,
                //             world_clone.clone(),
                //             children_layout_override_clone.clone(),
                //         );

                //         styles.values().fold(new_widget_state, |mut ws, x| {
                //             // x.shaping(&mut ws);
                //             ws.shape_of_use(x);
                //             // ws.shaping_use(x);
                //             ws
                //         })
                //     })
                //     .into_anchor()
            },
        );
        // let widget_state = layout_end.map(|&(trans, w, h)| WidgetState::new((w, h), trans));

        Self {
            id: ix.clone(),
            widget: Box::new(gel),
            // event_callbacks: VecDeque::default(),
            event_listener: EventListener::new(),
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
        ix: &IdStr,
        gel: GElement<Message>, //TODO use anchor instead
        edge_ctx: &StateAnchor<EdgeCtx>,
    ) -> Result<Self, GElement<Message>> {
        match gel {
            // Layer_(_) | Button_(_) | Text_(_) | GElement::Generic_(_) => Ok(Self::default()),
            GElement::Layer_(_) | GElement::Generic_(_) => Ok(Self::new(ix, gel, edge_ctx)),
            GElement::Builder_(_) => panic!("crate builder use builder is not supported"),
            GElement::Shaper_(_) | GElement::Event_(_) => Err(gel),
            // GElement::Refresher_(_) => Err(gel),
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
    // pub fn set_id(&mut self, id: IdStr) {
    //     self.id = id;
    // }

    // pub fn add_styles_string(&mut self, styles: &str) {
    //     self.layout_str += styles;
    // }

    /// Get a reference to the node builder widgets event callbacks.
    // #[must_use]
    // pub const fn event_callbacks(&self) -> &VecDeque<EventNode<Message>> {
    //     &self.event_callbacks
    // }

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
    pub const fn widget(&self) -> &GElement<Message> {
        //TODO use cow/beef

        &self.widget
    }

    pub fn widget_mut(&mut self) -> &mut GElement<Message> {
        &mut self.widget
    }

    pub fn add_event_callback(&mut self, event_callback: EventNode<Message>) {
        // self.event_callbacks.push_back(event_callback);
        self.event_listener
            .register_listener(event_callback.get_identify(), event_callback);
    }
    pub fn has_event_callback(&self) -> bool {
        !self.event_listener.event_callbacks().is_empty()
    }
    pub fn event_matching(
        &self,
        events_sa: &StateAnchor<Vector<emg_native::EventWithFlagType>>,
        cursor_position: &StateAnchor<Option<Pos>>,
    ) -> StateAnchor<EventMatchsDict<Message>> {
        let event_callbacks = self.event_listener.event_callbacks().clone();
        let cursor_position_clone = cursor_position.clone();
        let id = self.id.clone();

        (events_sa, &self.widget_state).then(move |events, state| {
            let _span = debug_span!("event_matching...", ?id).entered();

            // if events.is_empty() {
            //     return Anchor::constant(Dict::default());
            // }
            let size = state.size();

            //TODO don't do this many times
            let  event_filtered_matchs = events
                .iter()
                .map(|(ef, event)| (EventIdentify::from(*ef), event))
                .filter_map(|(ev_id, event)| {
                    //FIXME use filter_map instead,because filter can match multiple matches cb
                    event_callbacks.iter().find_map(|(&cb_ev_id, cb)| {
                        if ev_id.contains(cb_ev_id) {
                            Some((cb_ev_id, (event.clone(), cb.clone())))
                        } else {
                            None
                        }
                    })
                })
                .collect::<EventMatchsDict<Message>>();

            // let mut cb_matchs = event_callbacks
            //     .iter()
            //     .filter_map(|(e_name, cb)| {
            //         if let Some(x)=  e_str_s.iter().find(|(k,v)|) {
            //             Some((e_name.clone(), cb.clone()))
            //         } else {
            //             None
            //         }
            //     })
            //     .collect::<Dict<IdStr, Vector<EventNode<Message>>>>();

            // 提取 clicks
            let (click_group, other_event_cb_matchs): (EventMatchsDict<Message>, EventMatchsDict<Message>) =
                event_filtered_matchs.into_iter().partition(|(ev_id, _x)| {
                    ev_id.contains(EventIdentify::from(mouse::EventFlag::CLICK))
                });

                let id2 = id.clone();
                let cursor_position_clone2 = cursor_position_clone.clone();
            // let click_group = cb_matchs.remove_with_key("click");
            // let cursor_position_clone = cursor_position.clone();
            let clicked_a = click_group
                .into_iter()
                .map(|(cb_ev_id, (ev_, click_cb_vec))| {

                    let id3 = id2.clone();
                    (
                        &cursor_position_clone2,
                        &state.world,
                        &state.children_layout_override,
                    )
                        .map(move |c_pos, world, opt_layout_override| {

                            let id = id3.clone();

                            let ev = ev_.clone();

                            let click_cb_clone2 = click_cb_vec.clone();
                            let rect = Rect::from_origin_size((world.x, world.y), size);




                            c_pos.and_then( |pos| {
                                debug!(target:"event::click",?pos);

                                let _span = debug_span!("LayoutOverride",?id,func="event_matching").entered();


                                    debug!(target:"event::click",?world,?size,?rect,?pos);


                                let pos64 = pos.cast::<f64>();

                                if rect.contains(emg_native::renderer::Point::new(pos64.x, pos64.y))
                                {
                                    debug!("⭕️ rect contains pos");


                                    if let Some(layout_override) = opt_layout_override {
                                        debug!("⭕️ rect has layout_override");
                                        debug!("layout_override --> {:#?}",layout_override);

                                        if !layout_override.contains(&pos64) {

                                            debug!("❌ layout_override not contains pos ,not override, 🔔 ");
                                            Some((cb_ev_id, ev, click_cb_clone2))

                                        } else {

                                            debug!("⭕️ layout_override contains pos,override, 🔕 ");
                                            None
                                        }
                                    } else {
                                        debug!("❌ rect no layout_override, 🔔");
                                        Some((cb_ev_id, ev, click_cb_clone2))
                                    }
                                } else {
                                    debug!("❌ rect not contains pos, 🔕 ");

                                    None
                                }
                            })
                        }).into_anchor()
                })
                .collect::<Anchor<Vector<Option<(EventIdentify, Event, Vector<EventNode<Message>>)>>>>()
                .map(|clicked| {
                    clicked.clone()
                        .into_iter()
                        .flatten()
                        .collect::<Vector<_>>()
                });

                //TODO clicked_a can make dict_sa?
                clicked_a
                    .map(move |clicked| {

                        clicked.clone().into_iter().fold(other_event_cb_matchs.clone(),|cb_matchs_add_x,(cb_ev_id,ev,cb_vec)|{
                            cb_matchs_add_x.update_with(cb_ev_id, (ev,cb_vec), |(old_ev,mut old_cb_vec),(new_ev,new_cb_vec)|{
                                assert_eq!(old_ev, new_ev);
                                old_cb_vec.extend(new_cb_vec);
                                (old_ev,old_cb_vec)
                            })
                        })
                    })


        })
    }

    pub fn event_callbacks(&self) -> &Dict<EventIdentify, Vector<EventNode<Message>>> {
        self.event_listener.event_callbacks()
    }
}

#[cfg(all(feature = "gpu"))]
impl<Message> crate::Widget for NodeBuilderWidget<Message>
where
    Message: 'static,
    // Message: PartialEq + 'static + std::clone::Clone,
{
    type SceneCtxType = crate::SceneFrag;
    #[instrument(skip(self, ctx), name = "NodeBuilderWidget paint")]
    fn paint_sa(&self, ctx: &StateAnchor<crate::PaintCtx>) -> StateAnchor<Rc<Self::SceneCtxType>> {
        let id1 = self.id.clone();
        let id2 = self.id.clone();
        let opt_span = illicit::get::<Span>().ok();

        let span1 = opt_span.map_or_else(
            || info_span!("NodeBuilderWidget::paint_sa", id = %self.id),
            |span_| info_span!(parent:&*span_,"NodeBuilderWidget::paint_sa", id = %self.id),
        );
        let span2 = span1.clone();
        let span3 = span1.clone();

        let ctx_id = CtxIndex::new();
        let ctx_id2 = ctx_id.clone();

        let current_ctx =
            (ctx, &self.widget_state).map(move |incoming_ctx, current_widget_state| {
                // let id = id.clone();
                let _span = span1.clone().entered();
                info!(
                    parent: &span1,
                    "NodeBuilderWidget::paint-> (&ctx, &self.widget_state).map -> recalculating [{}]",
                    &id1
                );
                let mut incoming_ctx_mut = incoming_ctx.clone();
                // incoming_ctx_mut.save_assert(&ctx_id);
                incoming_ctx_mut.merge_widget_state(current_widget_state);


                // incoming_ctx_mut.transform(crate::renderer::Affine::translate((
                //     current_widget_state.translation.x * DPR,
                //     current_widget_state.translation.y * DPR,
                // )));
                incoming_ctx_mut
            });
        illicit::Layer::new()
            .offer(span3)
            .enter(|| self.widget.paint_sa(&current_ctx))
        // .map(move |out_scene| {
        //     info!(
        //         parent: &span2,
        //         " widget.paint end -> recalculating restore [{}]", &id2
        //     );
        //     let mut out_ctx_mut = out_ctx.clone();
        //     out_ctx_mut.restore_assert(&ctx_id2);
        //     out_ctx_mut
        // })
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
