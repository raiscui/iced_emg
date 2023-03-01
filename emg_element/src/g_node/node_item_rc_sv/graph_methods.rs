/*
 * @Author: Rais
 * @Date: 2022-09-07 14:20:32
 * @LastEditTime: 2023-03-01 17:59:44
 * @LastEditors: Rais
 * @Description:
 */

use std::{cell::RefCell, rc::Rc};

use emg::{edge_index_no_source, EdgeIndex, NodeIndex, Outgoing};
use emg_common::{
    im::{vector, OrdSet},
    IdStr, Pos, Vector,
};
use emg_layout::{EPath, EdgeItemNode};
use emg_native::{Event, EventWithFlagType, PaintCtx, Widget};
use emg_state::{Anchor, AnchorMultiAnchor, Dict, StateAnchor, StateMultiAnchor};
use tracing::{debug, debug_span};

use crate::{
    node_builder::{EventIdentify, EventMatchsDict},
    EventNode,
};

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
        let event_matchs: StateAnchor<Vector<Vector<EventMatchsDict<Message>>>> = self
            .borrow()
            .edges
            .watch()
            .filter_map_with_anchor(&root_eix_sa, move |root_eix, eix, _| {
                let borrow = self_clone2.borrow();
                let root_eix = root_eix.clone();

                let events = events.clone();
                let cursor_position_clone = cursor_position_clone.clone();

                eix.target_nix()
                    .and_then(|nix| borrow.get_node_item(nix))
                    .map(|item| {
                        let f: StateAnchor<Vector<EventMatchsDict<Message>>> = item
                            .paths_view_gel
                            .filter_map(move |ep, gel| {
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
                                            .event_matching(&events, &cursor_position_clone)
                                            .into_anchor(),
                                    )
                                })
                            })
                            .into();

                        f.into_anchor()
                    })
            })
            .into();
        let event_matchs: EventMatchsSa<Message> = event_matchs.map(|vv| {
            Dict::unions_with(vv.into_iter().cloned().flatten(), |mut old, new| {
                assert_eq!(old.0, new.0);
                old.1.append(new.1);
                old
            })
        });

        // let self_event_nodes = gel_rc_sa.then(move |gel| {
        //     //TODO 使用 判断event EventIdentify 类型 来优化

        //     let builder = gel.as_builder().unwrap();
        //     if builder.has_event_callback() {
        //         builder
        //             .event_matching(&events, &cursor_position_clone)
        //             .into_anchor()
        //     } else {
        //         //NOTE check if Panics , can not use constant, do Dict::new() in fn event_matching
        //         Anchor::constant(Dict::new())
        //     }
        // });

        // let children_event_matchs = self
        //     .get_out_going_event_callbacks(&NodeIndex::new(ix.clone()), events_sa, cursor_position)
        //     .into_iter()
        //     .collect::<Anchor<Vector<_>>>();

        // let event_matchs: EventMatchsSa<Message> =
        //     (self_event_nodes.anchor(), &children_event_matchs)
        //         .map(|self_event_nodes_dict, children| {
        //             debug!("child EventMatchsSa start");

        //             let mut self_add_children = children.clone();
        //             self_add_children.push_front(vector![self_event_nodes_dict.clone()]);

        //             let self_and_children_event_nodes_dict = Dict::unions_with(
        //                 self_add_children.into_iter().flatten(),
        //                 |mut old, new| {
        //                     assert_eq!(old.0, new.0);
        //                     old.1.append(new.1);
        //                     old
        //                 },
        //             );

        //             debug!("child EventMatchsSa end");

        //             self_and_children_event_nodes_dict
        //         })
        //         .into();

        let painter_clone = painter.clone();

        let ctx_sa = gel_rc_sa.then(move |gel| gel.paint_sa(&painter_clone).into_anchor());
        debug!("runtime prepare end");

        (event_matchs, ctx_sa)
    }

    // fn get_out_going_event_callbacks(
    //     &self,
    //     nix: &NodeIndex,
    //     events_sa: &StateAnchor<Vector<EventWithFlagType>>,
    //     cursor_position: &StateAnchor<Option<Pos>>,
    // ) -> Vector<Anchor<Vector<EventMatchsDict<Message>>>> {
    //     let out_goings = self.neighbors_consuming_iter(nix, Outgoing);
    //     out_goings.fold(Vector::default(), |mut vec, node| {
    //         let events = events_sa.clone();
    //         let cursor_position_clone = cursor_position.clone();
    //         let one_node_item = self.get_node_item(&node).unwrap();
    //         let event_cbs = one_node_item
    //             .paths_view_gel
    //             .filter_map(move |_k, gel| {
    //                 gel.as_builder()
    //                     .filter(|gel| gel.has_event_callback())
    //                     .map(|nb_widget| {
    //                         //TODO 使用 判断event EventIdentify 类型 来优化
    //                         nb_widget
    //                             .event_matching(&events, &cursor_position_clone)
    //                             .into_anchor()
    //                     })
    //             })
    //             .then(|dict| {
    //                 dict.values().collect::<Anchor<Vector<_>>>()
    //                 // .map(|vec_dict_event_nodes| {
    //                 //     Dict::unions_with(vec_dict_event_nodes.clone(), |mut old, new| {
    //                 //         old.append(new);
    //                 //         old
    //                 //     })
    //                 // })
    //             })
    //             .into_anchor();
    //         vec.push_back(event_cbs);
    //         let children_event_nodes =
    //             self.get_out_going_event_callbacks(&node, events_sa, cursor_position);
    //         vec.append(children_event_nodes);
    //         vec
    //     })
    // }
}
