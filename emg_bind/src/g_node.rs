/*
 * @Author: Rais
 * @Date: 2022-05-26 18:22:22
 * @LastEditTime: 2022-06-03 22:59:03
 * @LastEditors: Rais
 * @Description:
 */

use std::{cell::RefCell, rc::Rc};

use crate::{GElement, NodeBuilderWidget};

use super::N;
use emg::{GraphNodeMap, NodeEdgeCollect};
use emg_core::Vector;
use emg_layout::{EPath, EdgeItemNode, GraphEdgesDict};
use emg_refresh::RefreshForUse;
use emg_state::{Anchor, Dict, StateAnchor, StateMultiAnchor};
use tracing::{trace, trace_span};

#[derive(Clone)]
pub struct EmgNodeItem<Message, Ix>
where
    Message: 'static + Clone + std::cmp::PartialEq,
    Ix: std::clone::Clone + std::hash::Hash + std::cmp::Eq + std::default::Default,
    // Dict<EPath<Ix>, EmgNodeItem<Message, Ix>>: PartialEq,
{
    gel_sa: N<Message>,
    //TODO maybe indexSet
    paths_sa: StateAnchor<Vector<EPath<Ix>>>, //NOTE: has self
    incoming_eix_sa: StateAnchor<NodeEdgeCollect<Ix>>,
    outgoing_eix_sa: StateAnchor<NodeEdgeCollect<Ix>>,
    paths_view_gel_sa: StateAnchor<Dict<EPath<Ix>, GElement<Message>>>,
}
type GraphEdges<Ix> = StateAnchor<GraphEdgesDict<Ix>>;

impl<Message, Ix> EmgNodeItem<Message, Ix>
where
    Message: Clone + std::cmp::PartialEq,
    Ix: std::clone::Clone
        + std::hash::Hash
        + std::cmp::Eq
        + std::default::Default
        + std::cmp::Ord
        + std::convert::AsRef<str>
        + std::fmt::Display,
    // Dict<EPath<Ix>, EmgNodeItem<Message, Ix>>: PartialEq,
{
    pub fn new<E>(
        nix: Ix,
        gel_sa: N<Message>,
        incoming_eix_sa: StateAnchor<NodeEdgeCollect<Ix>>,
        outgoing_eix_sa: StateAnchor<NodeEdgeCollect<Ix>>,
        nodes: Rc<RefCell<GraphNodeMap<EmgNodeItem<Message, Ix>, Ix>>>,
        edges: GraphEdges<Ix>,
    ) -> Self {
        let nodes2 = nodes.clone();
        let paths_sa = incoming_eix_sa.then(move |ins| {
            let nix2 = nix.clone();

            ins.iter()
                .filter_map(|in_eix| in_eix.source_nix().as_ref())
                .filter_map(|source_nix| nodes2.borrow().get(source_nix.index()).cloned())
                .map(|p_node| p_node.item.paths_sa.clone())
                .map(|x| x.get_anchor())
                .collect::<Anchor<Vector<_>>>()
                .map(move |vv: &Vector<_>| {
                    let vvc = vv.clone();
                    vvc.into_iter()
                        .flatten()
                        .map(|x| x.link_ref(nix2.clone().into()))
                        .collect::<Vector<_>>()
                })
            //
        });

        let gel_sa2 = gel_sa.clone();

        //TODO children_path_map /paths_render_gel map

        let paths_view_gel_sa = (&paths_sa, &outgoing_eix_sa).then(move |paths, outs| {
            let current_nix = paths
                .last()
                .and_then(|x| x.last())
                .and_then(|x| x.target_nix().as_ref())
                .cloned()
                .unwrap();
            let current_nix2 = current_nix.clone();

            let children_view_gel_sa: StateAnchor<Dict<EPath<Ix>, GElement<Message>>> = outs
                .iter()
                .filter_map(|out_eix| out_eix.target_nix().as_ref())
                .filter_map(|target_nix| nodes.borrow().get(target_nix.index()).cloned())
                .map(move |child_node| {
                    let current_nix3 = current_nix2.clone();
                    child_node.item.paths_view_gel_sa.filter(move |path, _gel| {
                        &path
                            .last()
                            .and_then(|p| p.source_nix().as_ref())
                            .cloned()
                            .unwrap()
                            == &current_nix3
                    })
                })
                .map(|x| x.get_anchor())
                .collect::<Anchor<Vector<_>>>()
                .map(|v: &Vector<_>| Dict::unions(v.clone()))
                .into();

            let current_nix3 = current_nix.clone();

            let paths_view_gel_dict_sa = paths
                .clone()
                .into_iter()
                .map(|path| {
                    let current_nix4 = current_nix3.clone();

                    let path2 = path.clone();
                    let this_path_children_sa = children_view_gel_sa
                        .filter(move |k, _v| {
                            let mut for_current_ep = k.clone();
                            for_current_ep.pop_back();
                            for_current_ep == path2
                        })
                        .map(|d| d.values().cloned().collect::<Vec<_>>());
                    let children_no_cb_sa = this_path_children_sa.map(|this_path_children| {
                        this_path_children
                            .iter()
                            .filter(|gel| !gel.is_event_())
                            .cloned()
                            .collect::<Vec<_>>()
                    });
                    let event_callbacks_sa = this_path_children_sa.map(|this_path_children| {
                        this_path_children
                            .iter()
                            .filter(|gel| gel.is_event_())
                            .cloned()
                            .collect::<Vec<_>>()
                    });

                    let path3 = path.clone();

                    //TODO use filter
                    let styles_string_sa = edges.then(move |es| {
                        let path4 = path3.clone();

                        es.get(path3.last().unwrap())
                            .unwrap()
                            .item
                            .edge_nodes
                            .then(move |e_nodes| {
                                e_nodes
                                    .get(&path4)
                                    .and_then(EdgeItemNode::as_edge_data)
                                    .unwrap_or_else(|| {
                                        panic!("not find EdgeData for path:{}", &path4)
                                    })
                                    .styles_string
                                    .get_anchor()
                            })
                            .get_anchor()
                    });

                    let path4 = path.clone();

                    let view_gel_sa: StateAnchor<(EPath<Ix>, GElement<Message>)> = (
                        &gel_sa2,
                        &styles_string_sa,
                        &children_no_cb_sa,
                        &event_callbacks_sa,
                    )
                        .map(move |gel, edge_styles, children, event_callbacks| {
                            let mut gel_clone = gel.clone();
                            //TODO illicit::Layer path
                            for child in children {
                                gel_clone.refresh_for_use(child);
                            }

                            if let Ok(mut node_builder_widget) =
                                NodeBuilderWidget::<Message>::try_new_use(&gel_clone)
                            {
                                let _g = trace_span!("-> in NodeBuilderWidget").entered();
                                {
                                    trace!("NodeBuilderWidget::<Message>::try_from  OK");
                                    // node_builder_widget.set_id(format!("{}", cix));
                                    node_builder_widget.set_id(current_nix4.index().clone().into());

                                    //TODO use StateAnchor ? for child edge change
                                    trace!("edge::path:  {}", &path4);

                                    trace!("styles---------------> {}", &edge_styles);

                                    node_builder_widget.add_styles_string(edge_styles.as_str());

                                    if !event_callbacks.is_empty() {
                                        for callback in event_callbacks {
                                            //TODO maybe just directly push event
                                            node_builder_widget.refresh_for_use(callback);
                                        }
                                    }

                                    (
                                        path4.clone(),
                                        GElement::Builder_(
                                            Box::new(gel_clone),
                                            node_builder_widget,
                                        ),
                                    )
                                }
                            } else {
                                trace!(
                                    "NodeBuilderWidget::<Message>::try_from  error use:",
                                    // current_node_clone.borrow()
                                );
                                (path4.clone(), gel_clone)
                            }
                        });

                    view_gel_sa.get_anchor()
                })
                .collect::<Anchor<Vector<_>>>()
                .map(|x| Dict::<EPath<Ix>, GElement<Message>>::from_iter(x.clone()));

            paths_view_gel_dict_sa
        });

        Self {
            paths_sa,
            gel_sa,
            incoming_eix_sa,
            outgoing_eix_sa,
            paths_view_gel_sa,
        }
    }
}
