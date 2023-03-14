/*
 * @Author: Rais
 * @Date: 2022-09-07 14:20:32
 * @LastEditTime: 2023-03-14 10:15:48
 * @LastEditors: Rais
 * @Description:
 */

use std::{cell::RefCell, rc::Rc};

use emg::{edge_index_no_source, EdgeIndex};
use emg_common::{
    im::{self, vector},
    Pos, Vector,
};
use emg_layout::EPath;
use emg_native::{EventWithFlagType, PaintCtx, Widget};
use emg_state::{Dict, StateAnchor};
use tracing::{debug, debug_span};

use crate::node_builder::{EvMatch, EventMatchs};

use super::{EventMatchsSa, GraphType};
pub trait GraphMethods<Message> {
    type SceneCtx;
    fn runtime_prepare(
        &self,
        root_eix: StateAnchor<Option<EdgeIndex>>,
        painter: &StateAnchor<PaintCtx>,
        events_sa: &StateAnchor<Vector<EventWithFlagType>>,
        cursor_position: &StateAnchor<Option<Pos>>,
    ) -> (EventMatchsSa<Message>, StateAnchor<Rc<Self::SceneCtx>>);

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
    type SceneCtx = crate::SceneFrag;

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
        painter: &StateAnchor<PaintCtx>,
        events_sa: &StateAnchor<Vector<EventWithFlagType>>,
        cursor_position: &StateAnchor<Option<Pos>>,
    ) -> (EventMatchsSa<Message>, StateAnchor<Rc<Self::SceneCtx>>) {
        debug!("runtime prepare start");
        let root_eix_sa = opt_root_eix_sa.map(|x| x.clone().unwrap());
        let root_nix_sa = root_eix_sa.map(|x| x.target_nix().cloned().unwrap());
        let events = events_sa.clone();
        let cursor_position_clone = cursor_position.clone();

        let self_clone = self.clone();

        let gel_rc_sa = root_nix_sa.then(move |root| {
            self_clone
                .borrow()
                .get_node_item(root)
                .unwrap()
                //TODO 当前只能是 edge_index_no_source ,  需要支持 任意节点 任意路径 (通过给 gtree 添加 root 头 来给 节点增加一个 edge_index_no_source)
                .get_view_gelement_sa(&EPath::new(vector![edge_index_no_source(
                    root.index().clone()
                )]))
                .into_anchor()
        });

        // //NOTE 优势 : 后续不去 get 那些没有在 root下的 node. 新增变更 一个元素只执行一次, 劣势:过程复杂复杂
        // let this_tree_nix_s =
        //     self.edges
        //         .watch()
        //         .map_with_anchor(&root_eix_sa, |root_eix, eix, edge| {
        //             edge.item.edge_nodes.map(|dict| {
        //                 //TODO test code need to finally code
        //                 let ep = dict.keys().next().unwrap();
        //                 if ep.contains(root_eix) {
        //                     debug_assert!(ep.last_target().is_some());
        //                     //一个edge的 edge_nodes key (epath)  last_target 永远唯一
        //                     ep.last_target().cloned()
        //                 } else {
        //                     None
        //                 }
        //             })
        //         });
        let self_clone2 = self.clone();
        let event_matchs_pool = im::vector::RRBPool::<EvMatch<Message>>::new(8);
        let event_matchs: StateAnchor<Vector<Vector<EventMatchs<Message>>>> = self
            .borrow()
            .edges
            .watch()
            .filter_map_with_anchor(&root_eix_sa, move |root_eix, eix, _| {
                let _span = debug_span!(
                    "event_matching",
                    at = "graph edges changed or root eix changed",
                    %eix
                )
                .entered();
                let borrow = self_clone2.borrow();
                let root_eix = root_eix.clone();

                let events = events.clone();
                let cursor_position_clone = cursor_position_clone.clone();
                let event_matchs_pool = event_matchs_pool.clone();

                eix.target_nix()
                    .and_then(|nix| borrow.get_node_item(nix))
                    .map(|item| {
                        let f: StateAnchor<Vector<EventMatchs<Message>>> = item
                            .paths_view_gel
                            .filter_map(move |ep, gel| {
                                let _span = debug_span!(
                                    "event_matching",
                                    func="runtime_prepare",
                                    at = "paths_view_gel changed",
                                    %ep
                                )
                                .entered();
                                if !ep.contains(&root_eix) {
                                    return None;
                                }
                                //
                                gel.as_builder().and_then(|builder| {
                                    if !builder.has_event_callback() {
                                        return None;
                                    }
                                    //
                                    Some(
                                        builder
                                            .event_matching(
                                                &events,
                                                &cursor_position_clone,
                                                &event_matchs_pool,
                                            )
                                            .into_anchor(),
                                    )
                                })
                            })
                            .into();

                        f.into_anchor()
                    })
            })
            .into();
        // let event_matchs: EventMatchsSa<Message> = event_matchs.map(|vv| {
        //     // Dict::unions_with(vv.into_iter().flatten().cloned(), |mut old, new| {
        //     //     assert_eq!(old.0, new.0);
        //     //     old.1.append(new.1);
        //     //     old
        //     // })
        //     vv.clone().into_iter().flatten().flatten().collect()
        // });

        let painter_clone = painter.clone();

        let ctx_sa = gel_rc_sa.then(move |gel| gel.paint_sa(&painter_clone).into_anchor());
        debug!("runtime prepare end");

        (event_matchs, ctx_sa)
    }
}
