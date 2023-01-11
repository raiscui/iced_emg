/*
 * @Author: Rais
 * @Date: 2022-09-07 14:20:32
 * @LastEditTime: 2023-01-11 17:28:40
 * @LastEditors: Rais
 * @Description:
 */

use std::rc::Rc;

use emg::{edge_index_no_source, NodeIndex, Outgoing};
use emg_common::{vector, IdStr, Pos, Vector};
use emg_layout::EPath;
use emg_native::{EventWithFlagType, PaintCtx, Widget};
use emg_state::{Anchor, AnchorMultiAnchor, Dict, StateAnchor};
use tracing::debug;

use crate::node_builder::EventMatchsDict;

use super::{EventMatchsSa, GraphType};
pub trait GraphMethods<Message, Ix = IdStr> {
    type SceneCtx;
    fn runtime_prepare(
        &self,
        ix: &IdStr,
        ctx: &StateAnchor<PaintCtx>,
        events_sa: &StateAnchor<Vector<EventWithFlagType>>,
        cursor_position: &StateAnchor<Option<Pos>>,
    ) -> (EventMatchsSa<Message>, StateAnchor<Rc<Self::SceneCtx>>);

    #[allow(clippy::type_complexity)]
    fn get_out_going_event_callbacks(
        &self,
        nix: &NodeIndex<IdStr>,
        events_sa: &StateAnchor<Vector<EventWithFlagType>>,
        cursor_position: &StateAnchor<Option<Pos>>,
    ) -> Vector<Anchor<Vector<EventMatchsDict<Message>>>>;
}
impl<Message> GraphMethods<Message> for GraphType<Message>
where
    // Ix: std::hash::Hash
    //     + std::clone::Clone
    //     + std::cmp::Ord
    //     + std::default::Default
    //     + std::fmt::Debug,
    // SceneCtx: crate::renderer::SceneCtx + Clone + PartialEq + 'static,
    Message: 'static,
{
    type SceneCtx = crate::SceneFrag;

    // fn xx(&self,events_sa: &StateAnchor<Vector<Event>>) {
    //     let events = events_sa.clone();
    //     let fff =
    //         self.get_node_item_use_ix(ix)
    //             .unwrap()
    //             .get_view_gelement_sa(&EPath::<IdStr>::new(vector![edge_index_no_source(
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
    //                 .collect::<Anchor<Vector<(EPath<IdStr>, Rc<GElement<Message>>)>>>()
    //                 .map(
    //                     |x| -> Dict<EPath<IdStr>, Rc<GElement<Message>>> {
    //                         x.clone().into_iter().collect()
    //                     },
    //                 )
    //         })
    //         .filter_map(move |_k, gel| gel.as_builder().map(|nb| nb.event_matchs(&events)));
    // }

    #[tracing::instrument(skip(self, events_sa, ctx, cursor_position))]
    fn runtime_prepare(
        &self,
        ix: &IdStr,
        ctx: &StateAnchor<PaintCtx>,
        events_sa: &StateAnchor<Vector<EventWithFlagType>>,
        cursor_position: &StateAnchor<Option<Pos>>,
    ) -> (EventMatchsSa<Message>, StateAnchor<Rc<Self::SceneCtx>>) {
        debug!("runtime prepare start");
        let events = events_sa.clone();
        let cursor_position_clone = cursor_position.clone();
        let gel_rc_sa =
            self.get_node_item_use_ix(ix)
                .unwrap()
                .get_view_gelement_sa(&EPath::<IdStr>::new(vector![edge_index_no_source(
                    ix.clone()
                )]));
        let self_event_nodes = gel_rc_sa.then(move |gel| {
            //TODO 使用 判断event EventIdentify 类型 来优化

            let builder = gel.as_builder().unwrap();
            if builder.has_event_callback() {
                builder
                    .event_matching(&events, &cursor_position_clone)
                    .into_anchor()
            } else {
                //NOTE check if Panics , can not use constant, do Dict::new() in fn event_matching
                Anchor::constant(Dict::new())
            }
        });

        let children_event_matchs = self
            .get_out_going_event_callbacks(&NodeIndex::new(ix.clone()), events_sa, cursor_position)
            .into_iter()
            .collect::<Anchor<Vector<_>>>();

        let event_matchs: EventMatchsSa<Message> =
            (self_event_nodes.anchor(), &children_event_matchs)
                .map(|self_event_nodes_dict, children| {
                    debug!("child EventMatchsSa start");

                    let mut self_add_children = children.clone();
                    self_add_children.push_front(vector![self_event_nodes_dict.clone()]);

                    let self_and_children_event_nodes_dict = Dict::unions_with(
                        self_add_children.into_iter().flatten(),
                        |mut old, new| {
                            assert_eq!(old.0, new.0);
                            old.1.append(new.1);
                            old
                        },
                    );

                    debug!("child EventMatchsSa end");

                    self_and_children_event_nodes_dict
                })
                .into();

        let ctx_clone = ctx.clone();

        let ctx_sa = gel_rc_sa.then(move |gel| gel.paint_sa(&ctx_clone).into_anchor());
        debug!("runtime prepare end");

        (event_matchs, ctx_sa)
    }

    fn get_out_going_event_callbacks(
        &self,
        nix: &NodeIndex<IdStr>,
        events_sa: &StateAnchor<Vector<EventWithFlagType>>,
        cursor_position: &StateAnchor<Option<Pos>>,
    ) -> Vector<Anchor<Vector<EventMatchsDict<Message>>>> {
        let out_goings = self.neighbors_consuming_iter(nix, Outgoing);
        out_goings.fold(Vector::default(), |mut vec, node| {
            let events = events_sa.clone();
            let cursor_position_clone = cursor_position.clone();
            let one_node_item = self.get_node_item(&node).unwrap();
            let event_cbs = one_node_item
                .paths_view_gel
                .filter_map(move |_k, gel| {
                    gel.as_builder()
                        .filter(|gel| gel.has_event_callback())
                        .map(|nb_widget| {
                            //TODO 使用 判断event EventIdentify 类型 来优化
                            nb_widget
                                .event_matching(&events, &cursor_position_clone)
                                .into_anchor()
                        })
                })
                .then(|dict| {
                    dict.values().collect::<Anchor<Vector<_>>>()
                    // .map(|vec_dict_event_nodes| {
                    //     Dict::unions_with(vec_dict_event_nodes.clone(), |mut old, new| {
                    //         old.append(new);
                    //         old
                    //     })
                    // })
                })
                .into_anchor();
            vec.push_back(event_cbs);
            let children_event_nodes =
                self.get_out_going_event_callbacks(&node, events_sa, cursor_position);
            vec.append(children_event_nodes);
            vec
        })
    }
}
