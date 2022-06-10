/*
 * @Author: Rais
 * @Date: 2022-05-26 18:22:22
 * @LastEditTime: 2022-06-10 14:56:37
 * @LastEditors: Rais
 * @Description:
 */

mod index;

use std::{cell::RefCell, rc::Rc};

use crate::{GElement, NodeBuilderWidget};

use either::Either::{self, Left, Right};
use emg::{EdgeIndex, Graph, NodeEdgeCollect};
use emg_core::{vector, IdStr, Vector};
use emg_layout::{EPath, EdgeItemNode, EmgEdgeItem, GraphEdgesDict};
use emg_refresh::RefreshForUse;
use emg_state::{Anchor, CloneStateAnchor, Dict, StateAnchor, StateMultiAnchor};
use tracing::{trace, trace_span, warn};

pub type N<Message, Ix> = EmgNodeItem<Message, Ix>;
pub type NDate<Message> = StateAnchor<GElement<Message>>;
// pub type N< Message> = RefCell<GElement< Message>>;
pub type E<Ix> = EmgEdgeItem<Ix>;
pub type GraphType<Message, Ix = IdStr> = Graph<N<Message, Ix>, E<Ix>, Ix>;

// type PathDict<Ix> = Dict<DictIndex<EPath<Ix>>, bool>;
type PathDict<Ix> = Dict<EPath<Ix>, bool>;

#[derive(Clone)]
pub struct EmgNodeItem<Message, Ix = IdStr>
where
    Message: 'static + Clone + std::cmp::PartialEq,
    Ix: std::clone::Clone + std::hash::Hash + std::cmp::Eq + std::default::Default,
    // Dict<EPath<Ix>, EmgNodeItem<Message, Ix>>: PartialEq,
{
    gel_sa: NDate<Message>,
    //TODO maybe indexSet
    // paths_sa: StateAnchor<Vector<EPath<Ix>>>, //NOTE: has self
    paths_sa: StateAnchor<PathDict<Ix>>, //NOTE: has self
    // incoming_eix_sa: StateAnchor<NodeEdgeCollect<Ix>>,
    // outgoing_eix_sa: StateAnchor<NodeEdgeCollect<Ix>>,
    paths_view_gel_sa: StateAnchor<Dict<EPath<Ix>, StateAnchor<GElement<Message>>>>,
}

type CurrentPathChildrenEixGElSA<Message> = StateAnchor<(
    EdgeIndex<IdStr>,
    Either<GElement<Message>, GElement<Message>>,
)>;

impl<Message> EmgNodeItem<Message>
where
    Message: Clone + std::cmp::PartialEq,
    // Dict<EPath<Ix>, EmgNodeItem<Message, Ix>>: PartialEq,
{
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(
        nix: IdStr,
        gel_sa: NDate<Message>,
        incoming_eix_sa: StateAnchor<NodeEdgeCollect<IdStr>>,
        outgoing_eix_sa: StateAnchor<NodeEdgeCollect<IdStr>>,
        graph_rc: Rc<RefCell<GraphType<Message>>>,
    ) -> Self {
        let pool = graph_rc.borrow().nodes.;
        let graph_rc2 = graph_rc.clone();
        let nix2 = nix.clone();
        let paths_sa = incoming_eix_sa.then(move |ins| {
            ins.iter()
                .map(|in_eix| {
                    let res = in_eix.source_nix().as_ref().map_or(
                        Left(in_eix.clone()),
                        |self_source_nix| {
                            let nix2 = nix2.clone();

                            Right(
                                graph_rc2
                                    .borrow()
                                    .get_node_weight(self_source_nix)
                                    .unwrap()
                                    .paths_sa
                                    .get_anchor()
                                    .map(move |vec_e_path| {
                                        let vec_e_path_clone = vec_e_path.clone();
                                        vec_e_path_clone
                                            .into_iter()
                                            .map(|(ep, v)| (ep.link_ref(nix2.clone().into()), v))
                                            .collect::<PathDict<IdStr>>()
                                    }),
                            )
                        },
                    );
                    res.right_or_else(|no_source_self_eix| {
                        Anchor::constant(Dict::unit(EPath::new(vector![no_source_self_eix]), false))
                    })
                })
                .collect::<Anchor<Vector<_>>>()
                .map(|vd: &Vector<_>| Dict::unions(vd.clone()))
        });

        let graph_rc3 = graph_rc.clone();
        let nix3 = nix.clone();

        let children_view_gel_sa: StateAnchor<Dict<EPath<IdStr>, StateAnchor<GElement<Message>>>> =
            outgoing_eix_sa.then(move |outs| {
                outs.iter()
                    .filter_map(|out_eix| out_eix.target_nix().as_ref())
                    .filter_map(|out_target_nix| {
                        graph_rc3
                            .borrow()
                            .get_node_use_ix(out_target_nix.index())
                            .cloned()
                    })
                    .map(|child_node| {
                        let nix4 = nix3.clone();

                        child_node
                            .item
                            .paths_view_gel_sa
                            .filter(move |path, _gel| {
                                path.last()
                                    .and_then(|p| p.source_nix().as_ref())
                                    .map(emg::NodeIndex::index)
                                    .unwrap()//child source nix
                                    == &nix4
                            })
                            .get_anchor()
                    })
                    // .map(|x| x.get_anchor())
                    .collect::<Anchor<Vector<_>>>() //each edge-child vec --<  diff paths dict
                    .map(|v: &Vector<_>| Dict::unions(v.clone()))
            });
        // let children_count = children_view_gel_sa.map(Dict::len).get();
        // warn!("children count:{}", children_count);

        // @────────────────────────────────────────────────────────────────────────────────
        let gel_sa_clone1 = gel_sa.clone();
        let graph_rc3 = graph_rc.clone();
        let outgoing_eix_sa_clone = outgoing_eix_sa.clone();
        let paths_view_gel_sa = paths_sa.map_(move |current_path, _v| {
            let current_path2 = current_path.clone();
            let graph_rc4 = graph_rc3.clone();

            let this_path_children_sa = children_view_gel_sa
                .filter_map(move |k_child_path, v_child_gel_sa| {
                    let mut child_path_clone = k_child_path.clone();
                    //TODO check [current_child_ei] only one
                    let current_child_ei = child_path_clone.pop_back().unwrap();
                    let child_path_clone_popped = child_path_clone;
                    if child_path_clone_popped == current_path2 {
                        //
                        let graph_rc5 = graph_rc4.clone();
                        let v_child_gel_sa_clone = v_child_gel_sa.clone();
                        let gel_l_r: CurrentPathChildrenEixGElSA<Message> = v_child_gel_sa
                            .then(move |gel| {
                                // NOTE handle note_ref

                                if gel.is_node_ref_() {
                                    gel.as_node_ref_()
                                        .and_then(|str| {
                                            graph_rc5
                                                .borrow()
                                                .get_node_weight_use_ix(str)
                                                .map(|x| x.gel_sa.get_anchor())
                                        })
                                        .expect("expect get node id")
                                    // .map(move |g| g.clone())
                                } else {
                                    v_child_gel_sa_clone.get_anchor()
                                }
                            })
                            .map(move |gel| {
                                if gel.is_event_() {
                                    //Left event
                                    (current_child_ei.clone(), Left(gel.clone()))
                                } else {
                                    (current_child_ei.clone(), Right(gel.clone()))
                                }
                            });

                        Some(gel_l_r)
                        // let gel_l_r_clone = gel_l_r.clone();

                        // Some(gel_l_r.then(move |(k, x)| {
                        //     let kc = k.clone();
                        //     match x {
                        //         Left(_) => gel_l_r_clone.get_anchor(),
                        //         Right(r) => {
                        //             if r.is_node_ref_() {
                        //                 r.as_node_ref_()
                        //                     .and_then(|str| {
                        //                         graph_rc5
                        //                             .borrow()
                        //                             .get_node_weight_use_ix(str)
                        //                             .cloned()
                        //                     })
                        //                     .expect("expect get node id")
                        //                     .gel_sa
                        //                     .map(move |g| (kc.clone(), Right(g.clone())))
                        //                     .get_anchor()
                        //             } else {
                        //                 gel_l_r_clone.get_anchor()
                        //             }
                        //         }
                        //     }
                        // }))
                    } else {
                        None
                    }
                })
                .then(|children| {
                    // .map(|children| {
                    children
                        .values()
                        // .cloned()
                        .map(emg_state::StateAnchor::get_anchor)
                        .collect::<Anchor<Vector<_>>>()
                        .map(|v| {
                            v.clone().into_iter().collect::<Dict<
                                    EdgeIndex<IdStr>,
                                    Either<GElement<Message>, GElement<Message>>,
                                >>()
                        })
                    // .collect::<Dict<
                    //     EdgeIndex<IdStr>,
                    //     StateAnchor<Either<GElement<Message>, GElement<Message>>>,
                    // >>()
                });

            let path2 = current_path.clone();

            let styles_string_sa = graph_rc.borrow().edges.watch().then(move |es| {
                let path3 = path2.clone();

                es.get(path2.last().unwrap())
                    .unwrap()
                    .item
                    .edge_nodes
                    .then(move |e_nodes| {
                        e_nodes
                            .get(&path3)
                            .and_then(EdgeItemNode::as_edge_data)
                            .unwrap_or_else(|| panic!("not find EdgeData for path:{}", &path3))
                            .styles_string
                            .get_anchor()
                    })
                    .get_anchor()
            });

            let nix4 = nix.clone();
            let path3 = current_path.clone();
            let gel_sa_clone = gel_sa_clone1.clone();

            //TODO children Dict 细化 reduce, use diffitem 更新 gel_clone

            (
                &outgoing_eix_sa_clone,
                &this_path_children_sa,
                &gel_sa_clone,
                &styles_string_sa,
            )
                .map(move |out_eix_s, children, gel, edge_styles| {
                    let mut gel_clone = gel.clone();

                    for eix in out_eix_s {
                        if let Some(child_gel) =
                            children.get(eix).and_then(|child| child.as_ref().right())
                        {
                            gel_clone.refresh_for_use(child_gel);
                        }
                    }
                    // for child in children {
                    //     if let Some(child_gel) = child.as_ref().right() {
                    //         gel_clone.refresh_for_use(child_gel);
                    //     }
                    // }

                    if let Ok(mut node_builder_widget) =
                        NodeBuilderWidget::<Message>::try_new_use(&gel_clone)
                    {
                        let _g = trace_span!("-> in NodeBuilderWidget").entered();
                        {
                            trace!("NodeBuilderWidget::<Message>::try_from  OK");
                            // node_builder_widget.set_id(format!("{}", cix));
                            node_builder_widget.set_id(nix4.clone());

                            //TODO use StateAnchor ? for child edge change
                            trace!("edge::path:  {}", path3);

                            trace!("styles---------------> {}", &edge_styles);

                            node_builder_widget.add_styles_string(edge_styles.as_str());

                            // if !event_callbacks.is_empty() {
                            //     for callback in event_callbacks {
                            //         //TODO maybe just directly push event
                            //         node_builder_widget.refresh_for_use(callback);
                            //     }
                            // }

                            for eix in out_eix_s {
                                if let Some(event_gel) =
                                    children.get(eix).and_then(|child| child.as_ref().left())
                                {
                                    node_builder_widget.refresh_for_use(event_gel);
                                }
                            }

                            // for child in children {
                            //     if let Some(event_gel) = child.as_ref().left() {
                            //         node_builder_widget.refresh_for_use(event_gel);
                            //     }
                            // }

                            GElement::Builder_(Box::new(gel_clone), node_builder_widget)
                        }
                    } else {
                        trace!(
                            "NodeBuilderWidget::<Message>::try_from  error use:",
                            // current_node_clone.borrow()
                        );
                        gel_clone
                    }
                })
        });

        // let paths_view_gel_sa =
        //     (&paths_sa, &children_view_gel_sa).then(move |paths, children_view_gel| {
        //         // let current_nix = paths
        //         //     .last()
        //         //     .and_then(|x| x.last())
        //         //     .and_then(|x| x.target_nix().as_ref())
        //         //     .cloned()
        //         //     .unwrap();

        //         // let children_view_gel_sa: StateAnchor<Dict<EPath<IdStr>, GElement<Message>>> = outs
        //         //     .iter()
        //         //     .filter_map(|out_eix| out_eix.target_nix().as_ref())
        //         //     .filter_map(|target_nix| graph_rc3.borrow().nodes.get(target_nix.index()).cloned())
        //         //     .map(|child_node| {
        //         //         let nix3 = nix.clone();

        //         //         child_node.item.paths_view_gel_sa.filter(move |path, _gel| {
        //         //             path.last()
        //         //                 .and_then(|p| p.source_nix().as_ref())
        //         //                 .map(|x| x.index().clone())
        //         //                 .unwrap()
        //         //                 == nix3
        //         //         })
        //         //     })
        //         //     .map(|x| x.get_anchor())
        //         //     .collect::<Anchor<Vector<_>>>()
        //         //     .map(|v: &Vector<_>| Dict::unions(v.clone()))
        //         //     .into();

        //         paths
        //             .clone()
        //             .into_iter()
        //             .map(|path| {
        //                 let path2 = path.clone();
        //                 let this_path_children_sa = children_view_gel_sa
        //                     .filter(move |k, _v| {
        //                         let mut for_current_ep = k.clone();
        //                         for_current_ep.pop_back();
        //                         for_current_ep == path2
        //                     })
        //                     .map(|d| d.values().cloned().collect::<Vec<_>>());
        //                 let children_no_cb_sa = this_path_children_sa.map(|this_path_children| {
        //                     this_path_children
        //                         .iter()
        //                         .filter(|gel| !gel.is_event_())
        //                         .cloned()
        //                         .collect::<Vec<_>>()
        //                 });
        //                 let event_callbacks_sa = this_path_children_sa.map(|this_path_children| {
        //                     this_path_children
        //                         .iter()
        //                         .filter(|gel| gel.is_event_())
        //                         .cloned()
        //                         .collect::<Vec<_>>()
        //                 });

        //                 let path3 = path.clone();

        //                 //TODO use filter
        //                 let styles_string_sa = graph_rc.borrow().edges.watch().then(move |es| {
        //                     let path4 = path3.clone();

        //                     es.get(path3.last().unwrap())
        //                         .unwrap()
        //                         .item
        //                         .edge_nodes
        //                         .then(move |e_nodes| {
        //                             e_nodes
        //                                 .get(&path4)
        //                                 .and_then(EdgeItemNode::as_edge_data)
        //                                 .unwrap_or_else(|| {
        //                                     panic!("not find EdgeData for path:{}", &path4)
        //                                 })
        //                                 .styles_string
        //                                 .get_anchor()
        //                         })
        //                         .get_anchor()
        //                 });

        //                 let path4 = path;
        //                 let nix4 = nix.clone();

        //                 let view_gel_sa: StateAnchor<(EPath<IdStr>, GElement<Message>)> = (
        //                     &gel_sa2,
        //                     &styles_string_sa,
        //                     &children_no_cb_sa,
        //                     &event_callbacks_sa,
        //                 )
        //                     .map(move |gel, edge_styles, children, event_callbacks| {
        //                         let mut gel_clone = gel.clone();
        //                         //TODO illicit::Layer path
        //                         for child in children {
        //                             gel_clone.refresh_for_use(child);
        //                         }

        //                         if let Ok(mut node_builder_widget) =
        //                             NodeBuilderWidget::<Message>::try_new_use(&gel_clone)
        //                         {
        //                             let _g = trace_span!("-> in NodeBuilderWidget").entered();
        //                             {
        //                                 trace!("NodeBuilderWidget::<Message>::try_from  OK");
        //                                 // node_builder_widget.set_id(format!("{}", cix));
        //                                 node_builder_widget.set_id(nix4.clone());

        //                                 //TODO use StateAnchor ? for child edge change
        //                                 trace!("edge::path:  {}", &path4);

        //                                 trace!("styles---------------> {}", &edge_styles);

        //                                 node_builder_widget.add_styles_string(edge_styles.as_str());

        //                                 if !event_callbacks.is_empty() {
        //                                     for callback in event_callbacks {
        //                                         //TODO maybe just directly push event
        //                                         node_builder_widget.refresh_for_use(callback);
        //                                     }
        //                                 }

        //                                 (
        //                                     path4.clone(),
        //                                     GElement::Builder_(
        //                                         Box::new(gel_clone),
        //                                         node_builder_widget,
        //                                     ),
        //                                 )
        //                             }
        //                         } else {
        //                             trace!(
        //                                 "NodeBuilderWidget::<Message>::try_from  error use:",
        //                                 // current_node_clone.borrow()
        //                             );
        //                             (path4.clone(), gel_clone)
        //                         }
        //                     });

        //                 view_gel_sa.get_anchor()
        //             })
        //             .collect::<Anchor<Vector<_>>>()
        //             .map(|x| Dict::<EPath<IdStr>, GElement<Message>>::from_iter(x.clone()))
        //     });

        Self {
            gel_sa,
            paths_sa,
            incoming_eix_sa,
            outgoing_eix_sa,
            paths_view_gel_sa,
        }
    }
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    //TODO make no clone fn
    pub fn get_view_gelement_sa(&self, eix: &EPath<IdStr>) -> StateAnchor<GElement<Message>> {
        self.paths_view_gel_sa
            .get_with(|x| x.get(eix).unwrap().clone())
    }
}
