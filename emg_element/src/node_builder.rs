/*
 * @Author: Rais
 * @Date: 2022-08-18 18:05:52
 * @LastEditTime: 2023-03-14 12:27:16
 * @LastEditors: Rais
 * @Description:
 */

#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::ptr_eq)]
use either::Either::{self, Left, Right};
// ────────────────────────────────────────────────────────────────────────────────
use indented::indented;
use std::fmt::Write;
mod event_builder;

use emg_common::{
    im::{
        self,
        vector::{self, RRBPool},
    },
    mouse, IdStr, Pos, TypeName, Vector,
};
use emg_layout::EdgeCtx;
use emg_native::{event::EventIdentify, renderer::Rect, Event, WidgetState, EVENT_HOVER_CHECK};
use emg_shaping::EqShapingWithDebug;
use emg_state::{Anchor, Dict, StateAnchor, StateMultiAnchor};
use tracing::{debug, debug_span, info, info_span, instrument, Span};

use crate::GElement;
use std::{fmt::Display, rc::Rc, string::String};

pub use self::event_builder::*;

// ─────────────────────────────────────────────────────────────────────────────

#[allow(clippy::module_name_repetitions)]
pub struct NodeBuilderWidget<Message> {
    pub(crate) id: IdStr,
    //TODO : in areas heap
    pub(crate) widget: Box<GElement<Message>>,
    //TODO use vec deque
    // event_callbacks: VecDeque<EventNode<Message>>,
    pub(crate) event_listener: EventListener<Message>,
    // event_callbacks: Vector<EventNode<Message>>,
    // layout_str: String,
    // layout_end: (Translation3<NotNan<f64>>, NotNan<f64>, NotNan<f64>),
    pub(crate) widget_state: StateAnchor<WidgetState>,
}

impl<Message> Display for NodeBuilderWidget<Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let mut events = String::new();
        // for (i, m) in self.event_listener.event_callbacks.iter() {
        //     writeln!(events, "{}: {}", i, m)?
        // }

        let mut members = String::new();

        writeln!(members, "id: {}", self.id)?;

        writeln!(members, "widget: {}", &*self.widget)?;

        writeln!(members, "event_listener: {:?}", "no display")?;
        writeln!(members, "widget_state: {:?}", "no widget_state")?;

        write!(f, "NodeBuilderWidget {{\n{}}}", indented(members))
    }
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

impl<Message: 'static> std::fmt::Debug for NodeBuilderWidget<Message>
// where
//     Message: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let widget = if self.widget.is_none() {
        //     String::from("None")
        // } else {
        //     String::from("Option<Rc<dyn NodeBuilder<Message> + 'a>>")
        // };
        // let widget = String::from("GElement<Message>");

        f.debug_struct("NodeBuilderWidget")
            .field("id", &self.id)
            .field("widget", &self.widget)
            .field("event_listener", &self.event_listener)
            .field("widget_state", &self.widget_state)
            .finish()
    }
}

pub type EvMatch<Message> = (EventIdentify, Event, Vector<EventNode<Message>>);
pub type EventMatchs<Message> = Vector<EvMatch<Message>>;

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
                 world: world_sa,
                 children_layout_override: children_layout_override_sa,
                 ..
             }| {
                let styles_sa = styles_end.map_(|_k, v| v.get_anchor()).then(|x| {
                    x.clone()
                        .into_iter()
                        .collect::<Anchor<Dict<TypeName, Rc<dyn EqShapingWithDebug<WidgetState>>>>>(
                        )
                });

                //TODO 不要用 顺序pipe , 这样情况下 size trans改变 会 重新 进行全部 style 计算,使用 mut 保存 ws.
                (layout_end, children_layout_override_sa, world_sa)
                    .map(move |&(trans, w, h), children_layout_override, world| {
                        WidgetState::new(
                            (w, h),
                            trans,
                            Rc::new(*world),
                            Rc::new(children_layout_override.clone()),
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
            GElement::Layer_(_) | GElement::Generic_(_) => Ok(Self::new(ix, gel, edge_ctx)),
            GElement::Builder_(_) => panic!("crate builder use builder is not supported"),
            GElement::Shaper_(_) | GElement::Event_(_) => Err(gel),
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

    #[must_use]
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
        pool: &RRBPool<EvMatch<Message>>,
    ) -> StateAnchor<EventMatchs<Message>> {
        let event_callbacks = self.event_listener.event_callbacks().clone();
        //TODO move event_callbacks into sa map 不会变更, 是否考虑变更?
        let id = self.id.clone();
        let id2 = self.id.clone();
        let id3 = self.id.clone();
        let _span = debug_span!("event_matching", at = "event_matching pre run", ?id).entered();

        let widget_is_hover = (cursor_position, &self.widget_state).map(move |c_pos, state| {
            let size = state.size();
            let world = &state.world;
            let rect = Rect::from_origin_size((world.x as f64, world.y as f64), size);

            c_pos.is_some_and(|pos| {
                let _span = debug_span!("LayoutOverride",id=?id,func="event_matching").entered();

                debug!(target:"widget_is_hover",?world,?size,?rect,?pos);

                let pos_p = pos.cast::<f64>();

                if rect.contains(emg_native::renderer::Point::new(pos_p.x, pos_p.y)) {
                    debug!("⭕️ rect contains pos");

                    if let Some(layout_override) = &*state.children_layout_override {
                        debug!("⭕️ rect has layout_override");
                        // debug!("layout_override --> {:#?}", layout_override);

                        if !layout_override.contains(&pos_p) {
                            debug!("❌ layout_override not contains pos ,not override, 🔔 ");
                            true
                        } else {
                            debug!("⭕️ layout_override contains pos,override, 🔕 ");
                            false
                        }
                    } else {
                        debug!("❌ rect no layout_override, 🔔");
                        true
                    }
                } else {
                    debug!("❌ rect not contains pos, 🔕 ");

                    false
                }
            })
        });

        let pool = pool.clone();

        let matchs_step1 = (events_sa).map(move |events| {
            let mut ev_matchs = Vector::<EvMatch<Message>>::with_pool(&pool);

            //TODO don't do this many times  ,events change to Dict
            //已经根据event 事件 筛选出来的 callbacks
            events
                .iter()
                .flat_map(|(ef, event)| {
                    let ev_id = EventIdentify::from(*ef);

                    event_callbacks
                        .iter()
                        .filter_map(move |(cb_ev_id_wide, cb)| {
                            //ev_id 具体, cb_ev_id 宽泛
                            if ev_id.contains(cb_ev_id_wide) {
                                Some((ev_id, event.clone(), cb.clone()))
                                // debug!(target :"winit_event",id=?id3, ?intersects,?is_hover);
                            } else {
                                None
                            }

                            // ─────────────────────
                        })
                })
                // .flatten()
                .collect_into(&mut ev_matchs);
            debug!(target :"winit_event",id=?id2, ?ev_matchs);
            debug!(
                target : "winit_event",
                "==============================================================="
            );
            ev_matchs
        });

        //left is need hover check
        let matchs_step2_split = matchs_step1.map(|x| {
            x.iter()
                .cloned()
                .partition::<Vector<_>, _>(|(cb_ev_id_wide, _, _)| {
                    EVENT_HOVER_CHECK.intersects(cb_ev_id_wide)
                })
        });
        // let (a, b) = matchs_step2_split.split();
        let no_need_hover_check_matchs = matchs_step2_split.refmap(|x| &x.1);

        matchs_step2_split.then(move |(need_hover_ck, _)| {
            if need_hover_ck.is_empty() {
                matchs_step1.clone().into_anchor()
            } else {
                let matchs_step1 = matchs_step1.clone();
                let no_need_hover_check_matchs = no_need_hover_check_matchs.clone();
                widget_is_hover
                    .then(move |&is_hover| {
                        if is_hover {
                            matchs_step1.clone().into_anchor()
                        } else {
                            no_need_hover_check_matchs.clone().into_anchor()
                        }
                    })
                    .into_anchor()
            }
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
    #[instrument(skip(self, painter), name = "NodeBuilderWidget paint")]
    fn paint_sa(
        &self,
        painter: &StateAnchor<crate::PaintCtx>,
    ) -> StateAnchor<Rc<Self::SceneCtxType>> {
        let id1 = self.id.clone();
        let opt_span = illicit::get::<Span>().ok();

        let span1 = opt_span.map_or_else(
            || info_span!("NodeBuilderWidget::paint_sa", id = %self.id),
            |span_| info_span!(parent:&*span_,"NodeBuilderWidget::paint_sa", id = %self.id),
        );
        let span3 = span1.clone();

        let current_painter =
            (painter, &self.widget_state).map(move |incoming_painter, current_widget_state| {
                // let id = id.clone();
                let _span = span1.clone().entered();
                info!(
                    parent: &span1,
                    "NodeBuilderWidget::paint-> (&ctx, &self.widget_state).map -> recalculating [{}]",
                    &id1
                );
                let mut incoming_painter_mut = incoming_painter.clone();
                // incoming_ctx_mut.save_assert(&ctx_id);
                incoming_painter_mut.merge_widget_state(current_widget_state);


                // incoming_ctx_mut.transform(crate::renderer::Affine::translate((
                //     current_widget_state.translation.x * DPR,
                //     current_widget_state.translation.y * DPR,
                // )));
                incoming_painter_mut
            });
        illicit::Layer::new()
            .offer(span3)
            .enter(|| self.widget.paint_sa(&current_painter))
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
    use emg_common::im::vector;
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
