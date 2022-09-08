/*
 * @Author: Rais
 * @Date: 2022-09-07 14:20:32
 * @LastEditTime: 2022-09-08 16:36:43
 * @LastEditors: Rais
 * @Description:
 */

use emg::{edge_index_no_source, NodeIndex, Outgoing};
use emg_common::{vector, IdStr, Vector};
use emg_layout::EPath;
use emg_native::{Event, PaintCtx, Widget};
use emg_state::{Anchor, AnchorMultiAnchor, Dict, StateAnchor};

use crate::EventNode;

use super::{EventMatchsSa, GraphType};
pub trait GraphMethods<Message, RenderCtx, Ix = IdStr> {
    fn runtime_prepare(
        &self,
        ix: &IdStr,
        events_sa: &StateAnchor<Vector<Event>>,
        ctx: &StateAnchor<PaintCtx<RenderCtx>>,
    ) -> (EventMatchsSa<Message>, StateAnchor<PaintCtx<RenderCtx>>);

    #[allow(clippy::type_complexity)]
    fn get_out_going_event_callbacks(
        &self,
        events_sa: &StateAnchor<Vector<Event>>,
        nix: &NodeIndex<IdStr>,
    ) -> Vector<Anchor<Vector<Dict<IdStr, Vector<EventNode<Message>>>>>>;
}
impl<Message, RenderCtx> GraphMethods<Message, RenderCtx> for GraphType<Message, RenderCtx>
where
    // Ix: std::hash::Hash
    //     + std::clone::Clone
    //     + std::cmp::Ord
    //     + std::default::Default
    //     + std::fmt::Debug,
    RenderCtx: crate::RenderContext + Clone + PartialEq + 'static,
    Message: 'static,
{
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
    //                 .collect::<Anchor<Vector<(EPath<IdStr>, Rc<GElement<Message, RenderCtx>>)>>>()
    //                 .map(
    //                     |x| -> Dict<EPath<IdStr>, Rc<GElement<Message, RenderCtx>>> {
    //                         x.clone().into_iter().collect()
    //                     },
    //                 )
    //         })
    //         .filter_map(move |_k, gel| gel.as_builder().map(|nb| nb.event_matchs(&events)));
    // }

    #[tracing::instrument(skip(self, events_sa, ctx))]
    fn runtime_prepare(
        &self,
        ix: &IdStr,
        events_sa: &StateAnchor<Vector<Event>>,
        ctx: &StateAnchor<PaintCtx<RenderCtx>>,
    ) -> (EventMatchsSa<Message>, StateAnchor<PaintCtx<RenderCtx>>) {
        let events = events_sa.clone();

        let gel_rc_sa =
            self.get_node_item_use_ix(ix)
                .unwrap()
                .get_view_gelement_sa(&EPath::<IdStr>::new(vector![edge_index_no_source(
                    ix.clone()
                )]));
        let self_event_nodes = gel_rc_sa.then(move |gel| {
            gel.as_builder()
                .unwrap()
                .event_matchs(&events)
                .into_anchor()
        });

        let children_event_matchs = self
            .get_out_going_event_callbacks(events_sa, &NodeIndex::new(ix.clone()))
            .into_iter()
            .collect::<Anchor<Vector<_>>>();

        let event_matchs: EventMatchsSa<Message> =
            (self_event_nodes.anchor(), &children_event_matchs)
                .map(|s, children| {
                    let children_flatten = children.clone().into_iter().flatten();
                    let children_event_nodes_dict =
                        Dict::unions_with(children_flatten, |mut old, new| {
                            old.append(new);
                            old
                        });
                    s.clone()
                        .union_with(children_event_nodes_dict, |mut old, new| {
                            old.append(new);
                            old
                        })
                })
                .into();

        let ctx_clone = ctx.clone();

        let ctx_sa = gel_rc_sa.then(move |gel| gel.paint_sa(&ctx_clone).into_anchor());
        (event_matchs, ctx_sa)
    }

    fn get_out_going_event_callbacks(
        &self,
        events_sa: &StateAnchor<Vector<Event>>,
        nix: &NodeIndex<IdStr>,
    ) -> Vector<Anchor<Vector<Dict<IdStr, Vector<EventNode<Message>>>>>> {
        let out_goings = self.neighbors_consuming_iter(nix, Outgoing);
        out_goings.fold(Vector::default(), |mut vec, node| {
            let events = events_sa.clone();
            let node_item = self.get_node_item(&node).unwrap();
            let event_cbs = node_item
                .paths_view_gel
                .filter_map(move |_k, gel| {
                    gel.as_builder()
                        // .map(|nb_widget| nb_widget.event_callbacks().clone())
                        .map(|nb_widget| nb_widget.event_matchs(&events).into_anchor())
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
            let children_event_nodes = self.get_out_going_event_callbacks(events_sa, &node);
            vec.append(children_event_nodes);
            vec
        })
    }
}
