/*
 * @Author: Rais
 * @Date: 2022-06-18 12:53:14
 * @LastEditTime: 2022-08-23 00:05:30
 * @LastEditors: Rais
 * @Description:
 */
use std::{cell::RefCell, rc::Rc};

// use cfg_if::cfg_if;
use either::Either::{self, Left, Right};
use emg::{EdgeCollect, EdgeIndex, Graph};
use emg_common::{im::ordmap::OrdMapPool, vector, IdStr, Vector};
use emg_layout::{EPath, EdgeItemNode, EmgEdgeItem};
use emg_shaping::{ShapeOfUse, ShapingUse};
use emg_state::{
    Anchor, CloneStateAnchor, CloneStateVar, Dict, StateAnchor, StateMultiAnchor, StateVar,
};
use tracing::{debug, error, trace, trace_span, warn};
// use vec_string::VecString;

use crate::{GElement, NodeBuilderWidget};

use super::{EmgNodeItem, PathDict};

const POOL_SIZE: usize = 1;
// ────────────────────────────────────────────────────────────────────────────────

pub type GelType<Message,RenderContext> = Rc<GElement<Message,RenderContext>>;

pub type NItem<Message,RenderContext> = StateVar<StateAnchor<GelType<Message,RenderContext>>>;
pub type N<Message,RenderContext, Ix> = EmgNodeItem<NItem<Message,RenderContext>, GelType<Message,RenderContext>, Ix>;
pub type E<Ix> = EmgEdgeItem<Ix>;
pub type GraphType<Message,RenderContext, Ix = IdStr> = Graph<N<Message,RenderContext, Ix>, E<Ix>, Ix>;
// ────────────────────────────────────────────────────────────────────────────────
type GElEither<Message,RenderContext> = Either<GelType<Message,RenderContext>, GelType<Message,RenderContext>>;

type CurrentPathChildrenEixGElSA<Message,RenderContext> =
    StateAnchor<(EdgeIndex<IdStr>, GElEither<Message,RenderContext>)>;


impl<Message,RenderContext> EmgNodeItem<NItem<Message,RenderContext>, GelType<Message,RenderContext>>
where
    Message: 'static,
    RenderContext: 'static,
    // Dict<EPath<Ix>, EmgNodeItem<Message, Ix>>: PartialEq,
{
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(
        nix: IdStr,
        gel_sa: NItem<Message,RenderContext>,
        incoming_eix_sa: &StateAnchor<EdgeCollect<IdStr>>,
        outgoing_eix_sa: &StateAnchor<EdgeCollect<IdStr>>,
        graph_rc: Rc<RefCell<GraphType<Message,RenderContext>>>,
    ) -> Self {
        let graph_rc2 = graph_rc.clone();
        let nix2 = nix.clone();
        let paths_ord_map_pool_0: OrdMapPool<EPath<IdStr>, bool> = OrdMapPool::new(POOL_SIZE);

        let paths_sa = incoming_eix_sa.then(move |ins| {
            let ord_map_pool = paths_ord_map_pool_0.clone();
            ins.iter()
                .map(|in_eix| {
                    let res = in_eix.source_nix().as_ref().map_or(
                        Left(in_eix.clone()),
                        |self_source_nix| {
                            let nix2 = nix2.clone();
                            let ord_map_pool2 = ord_map_pool.clone();
                            Right(
                                graph_rc2
                                    .borrow()
                                    .get_node_item(self_source_nix)
                                    .unwrap()
                                    .paths_sa
                                    .map(move |vec_e_path| {
                                        // cfg_if! {
                                        //     if #[cfg(feature = "pool")]{

                                        //         let mut pd = PathDict::<IdStr>::with_pool(&ord_map_pool2);
                                        //         let vec_e_path_clone = vec_e_path.clone();
                                        //         vec_e_path_clone
                                        //             .into_iter()
                                        //             .map(|(ep, v)| (ep.link_ref(nix2.clone().into()), v))
                                        //             .collect_into(&mut pd);
                                        //         pd
                                        //     }else{
                                        //         let vec_e_path_clone = vec_e_path.clone();
                                        //         vec_e_path_clone
                                        //             .into_iter()
                                        //             .map(|(ep, v)| (ep.link_ref(nix2.clone().into()), v))
                                        //             .collect::<PathDict<IdStr>>()
                                        //     }
                                        // }
                                        let vec_e_path_clone = vec_e_path.clone();
                                        vec_e_path_clone
                                            .into_iter()
                                            .map(|(ep, v)| (ep.link_ref(nix2.clone().into()), v))
                                            .collect::<PathDict<IdStr>>()
                                    })
                                    .get_anchor(),
                            )
                        },
                    );
                    res.right_or_else(|no_source_self_eix| {
                        // cfg_if!{
                        //     if #[cfg(feature = "pool")]{
                        //         let mut pd = PathDict::<IdStr>::with_pool(&ord_map_pool);
                        //         pd.insert(EPath::new(vector![no_source_self_eix]), false);
                        //         Anchor::constant(pd)
                        //     }else{
                        //         Anchor::constant(Dict::<EPath<IdStr>, bool>::unit(EPath::new(vector![no_source_self_eix]), false))
                        //     }
                        // }
                        Anchor::constant(Dict::<EPath<IdStr>, bool>::unit(
                            EPath::new(vector![no_source_self_eix]),
                            false,
                        ))
                    })
                })
                .collect::<Anchor<Vector<_>>>()
                .map(move |vd: &Vector<_>| {
                    // cfg_if! {
                    //     if #[cfg(feature = "pool")]{
                    //         vd.clone()
                    //         .into_iter()
                    //         .fold(PathDict::<IdStr>::with_pool(&ord_map_pool), Dict::union)
                    //     }else{
                    //         PathDict::<IdStr>::unions(vd.clone())
                    //     }
                    // }
                    PathDict::<IdStr>::unions(vd.clone())
                })
        });

        let graph_rc3 = graph_rc.clone();
        let nix3 = nix.clone();

        let children_ord_map_pool_0: OrdMapPool<EPath<IdStr>, StateAnchor<GelType<Message, RenderContext>>> =
            OrdMapPool::new(POOL_SIZE);

        let children_view_gel_sv_sa: StateAnchor<
            Dict<EPath<IdStr>, StateAnchor<GelType<Message, RenderContext>>>,
        > = outgoing_eix_sa.then(move |outs| {
            let children_ord_map_pool = children_ord_map_pool_0.clone();
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
                // .map(|v: &Vector<_>| Dict::unions(v.clone()))
                .map(move |vd: &Vector<_>| {
                    // cfg_if!{
                    //     if #[cfg(feature = "pool")]{
                    //         vd.clone().into_iter().fold(
                    //             Dict::<EPath<IdStr>, StateAnchor<GelType<Message>>>::with_pool(&children_ord_map_pool),
                    //             Dict::union,
                    //         )

                    //     }else{
                    //         Dict::<EPath<IdStr>, StateAnchor<GelType<Message>>>::unions(vd.clone())
                    //     }
                    // }
                    Dict::<EPath<IdStr>, StateAnchor<GelType<Message, RenderContext>>>::unions(vd.clone())
                })
        });
        // let children_count = children_view_gel_sa.map(Dict::len).get();
        // warn!("children count:{}", children_count);

        // @────────────────────────────────────────────────────────────────────────────────
        let graph_rc3 = graph_rc.clone();
        let outgoing_eix_sa_clone = outgoing_eix_sa.clone();

        let children_either_ord_map_pool_0: OrdMapPool<EdgeIndex<IdStr>, GElEither<Message, RenderContext>> =
            OrdMapPool::new(POOL_SIZE);

        let paths_view_gel_sa = paths_sa.map_(move |current_path, _v| {
            let current_path2 = current_path.clone();
            let graph_rc4 = graph_rc3.clone();

            let children_either_ord_map_pool_1 = children_either_ord_map_pool_0.clone();

            let this_path_children_sa: StateAnchor<Dict<EdgeIndex<IdStr>, GElEither<Message, RenderContext>>> =
                children_view_gel_sv_sa
                    .filter_map(move |k_child_path, v_child_gel_sv_sa| {
                        let mut child_path_clone = k_child_path.clone();
                        //TODO check [current_child_ei] only one
                        let current_child_ei = child_path_clone.pop_back().unwrap();
                        let child_path_clone_popped = child_path_clone;
                        if child_path_clone_popped == current_path2 {
                            let current_path3 = current_path2.clone();
                            //
                            let graph_rc5 = graph_rc4.clone();
                            let v_child_gel_sa_clone = v_child_gel_sv_sa.clone();
                            let gel_l_r: CurrentPathChildrenEixGElSA<Message, RenderContext> = v_child_gel_sv_sa
                                .then(move |gel| {
                                    // NOTE handle note_ref

                                    debug_assert!(!gel.is_node_ref_());

                                    if gel.is_node_ref_() {
                                        //TODO remove this

                                        let refs = gel.as_node_ref_().unwrap();
                                        error!(
                                            "child-- is node ref:{} path:{}",
                                            refs, current_path3
                                        );

                                        gel.as_node_ref_()
                                            .and_then(|str| {
                                                graph_rc5.borrow().get_node_item_use_ix(str).map(
                                                    |x| {
                                                        x.gel_sa
                                                            .watch()
                                                            .get_anchor()
                                                            .then(|aa| aa.clone().into())
                                                    },
                                                )
                                            })
                                            .expect("expect get node id")
                                        // .map(move |g| g.clone())
                                    } else {
                                        v_child_gel_sa_clone.get_anchor()
                                    }
                                })
                                .map(move |gel| {
                                    // if gel.is_event_() {
                                    //     //Left event
                                    //     (current_child_ei.clone(), Left(gel.clone()))
                                    // } else {
                                    //     (current_child_ei.clone(), Right(gel.clone()))
                                    // }
                                        (current_child_ei.clone(), Right(gel.clone()))

                                });

                            Some(gel_l_r)
                        } else {
                            None
                        }
                    })
                    .then(move |children| {
                        // .map(|children| {
                        let children_either_ord_map_pool_2 = children_either_ord_map_pool_1.clone();

                        children
                            .values()
                            // .cloned()
                            .map(emg_state::StateAnchor::get_anchor)
                            .collect::<Anchor<Vector<_>>>()
                            .map(move |v| {
                                // cfg_if! {

                                //     if #[cfg(feature = "pool")]{
                                //         let mut dict = Dict::<EdgeIndex<IdStr>, GElEither<Message>>::with_pool(
                                //             &children_either_ord_map_pool_2
                                //         );
                                //         v.clone().into_iter().collect_into(&mut dict);
                                //         dict
                                //     }else{
                                //         v.clone().into_iter().collect::<Dict<EdgeIndex<IdStr>, GElEither<Message>>>()
                                //     }

                                // }
                                v.clone()
                                    .into_iter()
                                    .collect::<Dict<EdgeIndex<IdStr>, GElEither<Message, RenderContext>>>()
                            })
                    });

            let path2 = current_path.clone();
            //TODO check once edge change, edges recalculation?
            let styles_string_sa = graph_rc.borrow().edges.watch().then(move |es| {
                let path3 = path2.clone();

                es.get(path2.last().unwrap())
                    .unwrap()
                    .item
                    .edge_nodes
                    .then(move |e_nodes| {
                        // let all_paths =  e_nodes.keys().cloned().collect::<Vec<_>>().vec_string();

                        e_nodes
                            .get(&path3)
                            .and_then(EdgeItemNode::as_edge_data)
                            // .unwrap_or_else(|| panic!("not find EdgeData for path:{} \n allPaths:\n{}", &path3,&all_paths))
                            .unwrap_or_else(|| panic!("not find EdgeData for path:{}", &path3))
                            .styles_string
                            .get_anchor()
                    })
                    .get_anchor()
            });

            let nix4 = nix.clone();
            let path3 = current_path.clone();
            let graph_rc6 = graph_rc.clone();

            let gel_sa_no_sv = gel_sa.watch().then(move |g_sa| {
                let graph_rc7 = graph_rc6.clone();
                let g_sa2 = g_sa.clone();

                g_sa.then(move |gel| {
                    if gel.is_node_ref_() {
                        let refs = gel.as_node_ref_().unwrap();
                        error!("self is node ref:{} ", refs);
                        graph_rc7
                            .borrow()
                            .get_node_item_use_ix(refs)
                            .map(|x| x.gel_sa.watch().get_anchor().then(|aa| aa.clone().into()))
                            .unwrap()
                    } else {
                        g_sa2.clone().into()
                    }
                })
                .into()

                // g_sa.clone().into()
            });
            //TODO children Dict 细化 reduce, use diffitem 更新 gel_clone

            (
                &outgoing_eix_sa_clone,
                &this_path_children_sa,
                &gel_sa_no_sv,
                &styles_string_sa,
            )
                .map(move |out_eix_s, children, gel, edge_styles| {
                    let mut gel_clone = (**gel).clone();

                    //TODO 合并 此处和下方 两次 for (移动此处)
                    for eix in out_eix_s {
                        if let Some(child_gel) =
                            children.get(eix).and_then(|child| child.as_ref().right())
                        {
                            debug_assert!(!child_gel.is_node_ref_());
                            // if child_gel.is_node_ref_() {
                            //     let refs =child_gel.as_node_ref_().unwrap();
                            //     error!("child_gel is node ref:{} ",refs);
                            // }

                            gel_clone.shaping_use(child_gel.as_ref());
                        }
                    }

                    debug!("[combine view gel] gel_clone: {}", gel_clone);
                    // for child in children {
                    //     if let Some(child_gel) = child.as_ref().right() {
                    //         gel_clone.shape_of_use(child_gel);
                    //     }
                    // }
                    match NodeBuilderWidget::<Message, RenderContext>::try_new_use(gel_clone) {
                        Ok(mut node_builder_widget) => {
                            
                    let _g = trace_span!("-> in NodeBuilderWidget").entered();
                    {
                        trace!("[combine view gel] NodeBuilderWidget::<Message>::try_from  OK");
                        // node_builder_widget.set_id(format!("{}", cix));
                        node_builder_widget.set_id(nix4.clone());

                        //TODO use StateAnchor ? for child edge change
                        trace!("[combine view gel] edge::path:  {}", path3);
                        trace!("[combine view gel] styles---------------> {}", &edge_styles);
                        debug!("[combine view gel] edge::path:  {}", path3);
                        debug!("[combine view gel] styles---------------> {}", &edge_styles);

                        node_builder_widget.add_styles_string(edge_styles.as_str());

                        // if !event_callbacks.is_empty() {
                        //     for callback in event_callbacks {
                        //         //TODO maybe just directly push event
                        //         node_builder_widget.shape_of_use(callback);
                        //     }
                        // }

                        for eix in out_eix_s {
                            if let Some(event_gel) =
                                children.get(eix).and_then(|child| child.as_ref().left())
                            {
                                node_builder_widget.shape_of_use(event_gel.as_ref());
                            }
                        }

                        // for child in children {
                        //     if let Some(event_gel) = child.as_ref().left() {
                        //         node_builder_widget.shape_of_use(event_gel);
                        //     }
                        // }

                        Rc::new(GElement::Builder_(
                            // node_builder_widget.and_widget(gel_clone),
                            node_builder_widget
                        ))
                    }
                
                        },
                        Err(other_gel) => {
                            trace!(
                                "[combine view gel] NodeBuilderWidget::<Message>::try_from  error use:",
                                // current_node_clone.borrow()
                            );
                            Rc::new(other_gel)
                        },
                    }
                    
                })
        });

        Self {
            gel_sa,
            paths_sa,
            // incoming_eix_sa,
            // outgoing_eix_sa,
            paths_view_gel_sa,
        }
    }
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    //TODO make no clone fn
    pub fn get_view_gelement_sa(&self, eix: &EPath<IdStr>) -> StateAnchor<GelType<Message,RenderContext>> {
        self.paths_view_gel_sa
            .get_with(|x| x.get(eix).unwrap().clone())
    }
    pub fn set_gel_sa(&self, gel_sa: StateAnchor<GelType<Message,RenderContext>>) {
        self.gel_sa.set(gel_sa);
    }

    #[must_use]
    pub fn get_gel_rc_sa(&self) -> Rc<StateAnchor<Rc<GElement<Message,RenderContext>>>> {
        self.gel_sa.get_rc()
    }
}
