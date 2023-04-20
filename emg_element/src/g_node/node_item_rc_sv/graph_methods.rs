/*
 * @Author: Rais
 * @Date: 2022-09-07 14:20:32
 * @LastEditTime: 2023-04-14 11:03:17
 * @LastEditors: Rais
 * @Description:
 */

use std::{cell::RefCell, rc::Rc};

use emg::{edge_index_no_source, EdgeIndex, EdgePlugsCollect, NodeIndex};
use emg_common::{
    im::{
        self,
        vector::{self, RRBPool},
    },
    Pos, Vector,
};
use emg_layout::{EPath, EDGES_POOL_SIZE};
use emg_native::{EventWithFlagType, PaintCtx, Widget, GLOBAL_PENETRATE_EVENTS};
use emg_state::{Anchor, AnchorMultiAnchor, StateAnchor};
use tracing::{debug, debug_span};

use crate::{
    node_builder::{EvMatch, EventMatchs, HoverState},
    GelType, NodeBuilderWidget,
};

use super::{EventMatchsSa, GraphType};
// ─────────────────────────────────────────────────────────────────────────────

pub trait GraphMethods<Message> {
    type SceneCtx;
    fn runtime_prepare(
        &self,
        root_eix: StateAnchor<Option<EdgeIndex>>,
        painter: StateAnchor<PaintCtx>,
        events_sa: Anchor<Vector<EventWithFlagType>>,
        cursor_position: &StateAnchor<Option<Pos>>,
    ) -> (EventMatchsSa<Message>, StateAnchor<Rc<Self::SceneCtx>>);

    fn get_ep_out_vg(
        &self,
        parent_ep: EPath,
        nix: &NodeIndex,
    ) -> (
        EPath,
        StateAnchor<EdgePlugsCollect>,
        StateAnchor<GelType<Message>>,
    );

    // #[allow(clippy::type_complexity)]
    // fn get_out_going_event_callbacks(
    //     &self,
    //     nix: &NodeIndex,
    //     events_sa: &StateAnchor<Vector<EventWithFlagType>>,
    //     cursor_position: &StateAnchor<Option<Pos>>,
    // ) -> Vector<Anchor<Vector<EventMatchsDict<Message>>>>;
}
impl<Message> GraphMethods<Message> for Rc<RefCell<GraphType<Message>>>
where
    // SceneCtx: crate::renderer::SceneCtx + Clone + PartialEq + 'static,
    Message: 'static,
{
    type SceneCtx = crate::renderer::SceneFrag;

    // fn xx(&self,events_sa: &StateAnchor<Vector<Event>>) {
    //     let events = events_sa.clone();
    //     let fff =
    //         self.get_node_item_use_ix(ix)
    //             .unwrap()
    //             .get_view_gelement_sa(&EPath::new(vector![edge_index_no_source(
    //                 ix.clone()
    //             )]));
    //     let xx = self
    //         .get_node_item_use_ix(ix)
    //         .unwrap()
    //         .paths_view_gel_sa
    //         .then(|dict| {
    //             dict.iter()
    //                 .map(|(k, v)| {
    //                     let k_c = k.clone();
    //                     v.map(move |vv| (k_c.clone(), vv.clone())).into_anchor()
    //                 })
    //                 .collect::<Anchor<Vector<(EPath, Rc<GElement<Message>>)>>>()
    //                 .map(
    //                     |x| -> Dict<EPath, Rc<GElement<Message>>> {
    //                         x.clone().into_iter().collect()
    //                     },
    //                 )
    //         })
    //         .filter_map(move |_k, gel| gel.as_builder().map(|nb| nb.event_matchs(&events)));
    // }

    #[tracing::instrument(skip_all)]
    fn runtime_prepare(
        &self,
        opt_root_eix_sa: StateAnchor<Option<EdgeIndex>>,
        painter: StateAnchor<PaintCtx>,
        events_sa: Anchor<Vector<EventWithFlagType>>,
        cursor_position: &StateAnchor<Option<Pos>>,
    ) -> (EventMatchsSa<Message>, StateAnchor<Rc<Self::SceneCtx>>) {
        debug!("runtime prepare start");
        let root_eix_sa = opt_root_eix_sa.map(|x| x.clone().expect("root eix must exits"));
        let root_nix_sa = root_eix_sa.map(|x| {
            x.target_nix()
                .cloned()
                .expect("must has target,that is root it self ")
        });

        let self_clone2 = self.clone();
        let self_clone3 = self.clone();

        let gel_rc_sa = root_nix_sa.then(move |root| {
            self_clone2
                .borrow()
                .get_node_item(root)
                .expect("root must exits")
                //TODO 当前只能是 edge_index_no_source ,  需要支持 任意节点 任意路径 (通过给 gtree 添加 root 头 来给 节点增加一个 edge_index_no_source)
                .get_view_gelement_sa(&EPath::new(im::vector![edge_index_no_source(
                    root.index().clone()
                )]))
                .into_anchor()
        });
        let cursor_position = cursor_position.clone();

        let (_remain_events, event_matchs) = root_nix_sa
            .then(move |root| {
                let events_sa = events_sa.clone();
                let cursor_position = cursor_position.clone();

                let self_clone3 = self_clone3.clone();

                let (epath, out_targets_sa, maybe_builder_sa) =
                    self_clone3.get_ep_out_vg(EPath::new_uncheck(im::vector![]), root);
                maybe_builder_sa.anchor().then(move |gel| {
                    let builder = gel.as_builder().cloned().expect("root must builder");
                    let event_matchs_pool = im::vector::RRBPool::<EvMatch<Message>>::new(8);

                    get_event_match_cbs(
                        self_clone3.clone(),
                        event_matchs_pool,
                        events_sa.clone(),
                        cursor_position.clone(),
                        builder,
                        out_targets_sa.clone(),
                        epath.clone(),
                    )
                })
            })
            .split();

        // let edges_sa = self.borrow().edges.watch();

        // let event_matchs: StateAnchor<Vector<Vector<EventMatchs<Message>>>> = edges_sa
        //     .filter_map_with_anchor(EDGES_POOL_SIZE, &root_eix_sa, move |root_eix, eix, _| {
        //         let _span = debug_span!(
        //             "event_matching",
        //             at = "graph edges changed or root eix changed",
        //             %eix
        //         )
        //         .entered();
        //         let borrow = self_clone2.borrow();
        //         let root_eix = root_eix.clone();

        //         let events = events.clone();
        //         let cursor_position_clone = cursor_position_clone.clone();
        //         let event_matchs_pool = event_matchs_pool.clone();

        //         eix.target_nix()
        //             .and_then(|nix| borrow.get_node_item(nix))
        //             .map(|item| {
        //                 let f: StateAnchor<Vector<EventMatchs<Message>>> = item
        //                     .paths_view_gel
        //                     .filter_map(1, move |ep, gel| {
        //                         let _span = debug_span!(
        //                             "event_matching",
        //                             func="runtime_prepare",
        //                             at = "paths_view_gel changed",
        //                             %ep
        //                         )
        //                         .entered();
        //                         if !ep.contains(&root_eix) {
        //                             return None;
        //                         }
        //                         //
        //                         gel.as_builder().and_then(|builder| {
        //                             if !builder.has_event_callback() {
        //                                 return None;
        //                             }
        //                             //
        //                             Some(builder.event_matching(
        //                                 &events,
        //                                 &cursor_position_clone,
        //                                 &event_matchs_pool,
        //                             ))
        //                         })
        //                     })
        //                     .into();

        //                 f.into_anchor()
        //             })
        //     })
        //     .into();

        let ctx_sa = gel_rc_sa.then(move |gel| gel.paint_sa(&painter).into_anchor());
        debug!("runtime prepare end");

        (event_matchs, ctx_sa)
    }

    fn get_ep_out_vg(
        &self,
        parent_ep: EPath,
        nix: &NodeIndex,
    ) -> (
        EPath,
        StateAnchor<EdgePlugsCollect>,
        StateAnchor<GelType<Message>>,
    ) {
        let epath = parent_ep.link(nix.clone());

        let borrow = self.borrow();
        let node = borrow.get_node(nix).expect("node must exits");
        let out_targets_sa = node.outgoing().watch();

        let epath2 = epath.clone();
        let epath3 = epath.clone();

        let maybe_builder_sa = node
            .item
            .paths_view_gel
            .filter(1, move |ep, _| ep == &epath3)
            .map(move |x| x.get(&epath2).cloned().expect("must exits"));

        (epath, out_targets_sa, maybe_builder_sa)
    }
}

fn get_event_match_cbs<Message: 'static>(
    g: Rc<RefCell<GraphType<Message>>>,
    event_matchs_pool: RRBPool<EvMatch<Message>>,
    events_sa: Anchor<Vector<EventWithFlagType>>,
    cursor_position: StateAnchor<Option<Pos>>,
    builder: NodeBuilderWidget<Message>,
    out_targets_sa: StateAnchor<EdgePlugsCollect>,
    epath: EPath,
) -> Anchor<(Vector<emg_native::EventWithFlagType>, EventMatchs<Message>)> {
    let id = epath.last_target().cloned().unwrap();

    let _span = debug_span!(
        "event_matching",
        at = "pre run",
        func = "get_event_match_cbs",
        ?id
    )
    .entered();

    //    let (new_events_sa,children_ev_matchs)=
    out_targets_sa.anchor().then(move |outs| {
        let id = id.clone();
        let _span = debug_span!(
            "event_matching",
            at = "in then : out_targets_sa",
            func = "get_event_match_cbs",
            info = "target 出节点变化",
            ?id
        )
        .entered();

        let event_matchs_pool2 = event_matchs_pool.clone();
        let g = g.clone();
        let new_event = events_sa.clone();
        let cursor_position = cursor_position.clone();
        let builder = builder.clone();

        outs.iter()
            .rev()
            .flat_map(|out| {
                out.target_nix().map(|t_nix| {
                    let (ep, out_targets_sa, maybe_builder_sa) =
                        g.get_ep_out_vg(epath.clone(), t_nix);
                    // let out_and_gel =
                    //     (out_targets_sa.anchor(), maybe_builder_sa.anchor())
                    //         .map(|o, g| (o.clone(), g.clone()));
                    // (epath, out_and_gel)
                    let outs = maybe_builder_sa.anchor().map(move |gel| {
                        gel.as_builder()
                            .map(|builder| (ep.clone(), out_targets_sa.clone(), builder.clone()))
                    });
                    outs
                })
            })
            .collect::<Anchor<Vector<_>>>()
            .then(move |v| {
                let id = id.clone();

                let _span = debug_span!(
                    "event_matching",
                    at = "in then : collect 出节点ep,targets,gel-> Anchor<Vector<Option.... ",
                    func = "get_event_match_cbs",
                    info = "epath变化,target 出节点变化,view gel变化",
                    ?id
                )
                .entered();

                let new_event = new_event.clone();
                let cursor_position = cursor_position.clone();
                let event_matchs_pool2 = event_matchs_pool2.clone();

                //TODO 支持 event.stopPropagation 直接执行本级别 event_matching  而不进行子node event_matching

                let (children_latest_evs, mut children_cbs_vav) = v.iter().flatten().fold(
                    //TODO pool
                    (new_event, Vector::new()),
                    |(nev, mut cbs), (ep, out_t, child_builder)| {
                        let (new_evs, ev_matchs) = get_event_match_cbs(
                            g.clone(),
                            event_matchs_pool2.clone(),
                            nev,
                            cursor_position.clone(),
                            child_builder.clone(),
                            out_t.clone(),
                            ep.clone(),
                        )
                        .split();
                        cbs.push_back(ev_matchs);
                        (new_evs, cbs)
                    },
                );

                let latest_evs = if builder.has_event_callback() {
                    let (latest_evs, node_cbs) = builder
                        .event_matching(children_latest_evs, cursor_position, event_matchs_pool2)
                        .split();

                    children_cbs_vav.push_back(node_cbs);
                    latest_evs
                } else {
                    let hover_state =
                        builder.hover_state_check(&children_latest_evs, &cursor_position);

                        let new_events_no_hover =  hover_state.anchor().then(move |hs| {

                            let id = id.clone();


                            match hs {
                                HoverState::Hover => children_latest_evs.map(move |evs| {
                                    let _span = debug_span!(
                                        "event_matching",
                                        at = "events_sa hover filter - Hover , will remove all need EVENT_HOVER_CHECK events",
                                        events = ?evs,
                                        ?id
                                    )
                                    .entered();

                                    evs.iter()
                                        .filter(|(eid, ev)| {
                                            let r = GLOBAL_PENETRATE_EVENTS.read();
                                            let pass  = r.opt_involve_any_on(eid);
                                            let can_keep = match pass {
                                                Some(is_keep) => is_keep,//keep 就是穿透,保留
                                                None => true,
                                            };
                                            if !can_keep{
                                                debug!(?can_keep,?ev,"no cb node ev filter----");
                                            }

                                            can_keep

                                        })
                                        .cloned()
                                        .collect::<Vector<_>>()
                                }),
                                HoverState::NotHover | HoverState::HoverOverride => children_latest_evs.clone(),
                            }

                        });

                        new_events_no_hover


                };

                let all_cbs: Anchor<Vector<Vector<_>>> = children_cbs_vav.into();

                (&latest_evs, &all_cbs).map(|ev, cbs| {
                    (
                        ev.clone(),
                        cbs.clone().into_iter().flatten().collect::<Vector<_>>(),
                    )
                })
            })
    })
}
